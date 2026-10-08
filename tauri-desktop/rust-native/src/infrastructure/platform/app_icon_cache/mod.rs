//! Memory + disk cache in front of an [`AppIconSource`].

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::application::ports::app_icon::AppIconResolver;

/// PNG bytes plus the source app bundle's mtime token (seconds).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FetchedIcon {
    pub png: Vec<u8>,
    pub mtime_secs: u64,
}

/// Cold source that talks to the OS (or a test double).
pub trait AppIconSource: Send + Sync {
    /// Current app path mtime for `bundle_id`, if the app can be located.
    fn probe_mtime(&self, bundle_id: &str) -> Option<u64>;

    /// Fetch PNG bytes and the mtime token used for disk invalidation.
    fn fetch(&self, bundle_id: &str) -> Option<FetchedIcon>;
}

#[derive(Debug, Clone)]
enum MemoryEntry {
    Hit {
        data_url: String,
        mtime_secs: u64,
    },
    /// Negative cache (session only; not written to disk).
    Miss,
}

/// Resolves icons with memory → disk → source tiering.
pub struct CachedAppIconResolver<S: AppIconSource> {
    source: S,
    cache_dir: PathBuf,
    memory: Mutex<HashMap<String, MemoryEntry>>,
}

impl<S: AppIconSource> CachedAppIconResolver<S> {
    pub fn new(source: S, cache_dir: PathBuf) -> Self {
        Self {
            source,
            cache_dir,
            memory: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl<S: AppIconSource> AppIconResolver for CachedAppIconResolver<S> {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        if bundle_id.is_empty() {
            return None;
        }

        let current_mtime = self.source.probe_mtime(bundle_id);

        if let Ok(memory) = self.memory.lock() {
            if let Some(entry) = memory.get(bundle_id) {
                match entry {
                    MemoryEntry::Hit {
                        data_url,
                        mtime_secs,
                    } => {
                        if current_mtime == Some(*mtime_secs) {
                            return Some(data_url.clone());
                        }
                    }
                    MemoryEntry::Miss => {
                        current_mtime?;
                    }
                }
            }
        }

        if let Some(mtime_secs) = current_mtime {
            if let Some(png) = load_disk_png(&self.cache_dir, bundle_id, mtime_secs) {
                let data_url = png_data_url(&png);
                store_memory_hit(&self.memory, bundle_id, data_url.clone(), mtime_secs);
                return Some(data_url);
            }
        }

        match self.source.fetch(bundle_id) {
            Some(fetched) => {
                let _ = store_disk(&self.cache_dir, bundle_id, &fetched);
                let data_url = png_data_url(&fetched.png);
                store_memory_hit(
                    &self.memory,
                    bundle_id,
                    data_url.clone(),
                    fetched.mtime_secs,
                );
                Some(data_url)
            }
            None => {
                store_memory_miss(&self.memory, bundle_id);
                None
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct DiskMeta {
    #[serde(rename = "mtimeSecs")]
    mtime_secs: u64,
}

fn sanitize_bundle_id(bundle_id: &str) -> String {
    bundle_id
        .chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '\0' => '_',
            other => other,
        })
        .collect()
}

fn png_path(cache_dir: &Path, bundle_id: &str) -> PathBuf {
    cache_dir.join(format!("{}.png", sanitize_bundle_id(bundle_id)))
}

fn meta_path(cache_dir: &Path, bundle_id: &str) -> PathBuf {
    cache_dir.join(format!("{}.json", sanitize_bundle_id(bundle_id)))
}

fn load_disk_png(cache_dir: &Path, bundle_id: &str, expected_mtime: u64) -> Option<Vec<u8>> {
    let meta_bytes = fs::read(meta_path(cache_dir, bundle_id)).ok()?;
    let meta: DiskMeta = serde_json::from_slice(&meta_bytes).ok()?;
    if meta.mtime_secs != expected_mtime {
        return None;
    }
    let png = fs::read(png_path(cache_dir, bundle_id)).ok()?;
    if png.is_empty() {
        return None;
    }
    Some(png)
}

fn store_disk(cache_dir: &Path, bundle_id: &str, fetched: &FetchedIcon) -> std::io::Result<()> {
    fs::create_dir_all(cache_dir)?;
    let png_target = png_path(cache_dir, bundle_id);
    let meta_target = meta_path(cache_dir, bundle_id);
    atomic_write(&png_target, &fetched.png)?;
    let meta = DiskMeta {
        mtime_secs: fetched.mtime_secs,
    };
    let meta_bytes = serde_json::to_vec(&meta).map_err(std::io::Error::other)?;
    atomic_write(&meta_target, &meta_bytes)?;
    Ok(())
}

fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("cache.bin");
    let tmp = parent.join(format!("{file_name}.{}.tmp", std::process::id()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn store_memory_hit(
    memory: &Mutex<HashMap<String, MemoryEntry>>,
    bundle_id: &str,
    data_url: String,
    mtime_secs: u64,
) {
    if let Ok(mut guard) = memory.lock() {
        guard.insert(
            bundle_id.to_string(),
            MemoryEntry::Hit {
                data_url,
                mtime_secs,
            },
        );
    }
}

fn store_memory_miss(memory: &Mutex<HashMap<String, MemoryEntry>>, bundle_id: &str) {
    if let Ok(mut guard) = memory.lock() {
        guard.insert(bundle_id.to_string(), MemoryEntry::Miss);
    }
}

fn png_data_url(png: &[u8]) -> String {
    format!("data:image/png;base64,{}", base64_encode(png))
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(input: &[u8]) -> String {
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;

        out.push(BASE64_ALPHABET[((triple >> 18) & 0x3F) as usize] as char);
        out.push(BASE64_ALPHABET[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            out.push(BASE64_ALPHABET[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
        if chunk.len() > 2 {
            out.push(BASE64_ALPHABET[(triple & 0x3F) as usize] as char);
        } else {
            out.push('=');
        }
    }
    out
}

/// File mtime as whole seconds since UNIX epoch (for cache tokens).
pub fn file_mtime_secs(path: &Path) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    Some(
        modified
            .duration_since(SystemTime::UNIX_EPOCH)
            .ok()?
            .as_secs(),
    )
}

#[cfg(test)]
mod tests;
