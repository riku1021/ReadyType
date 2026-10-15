//! Memory + disk cache in front of an [`AppIconSource`].
//!
//! Resolved values are **filesystem paths** to cached PNG files (not data URLs),
//! so the WebView can load them via Tauri's asset protocol.

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::SystemTime,
};

use serde::{Deserialize, Serialize};

use crate::application::ports::app_icon::{AppIconResolver, ResolvedAppIcon};

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

/// Supplies a token that changes when system-rendered icon appearance changes.
pub trait IconStyleSource: Send + Sync {
    fn token(&self) -> String;
}

/// Default style source: empty token (no appearance-based invalidation).
pub struct EmptyIconStyle;

impl IconStyleSource for EmptyIconStyle {
    fn token(&self) -> String {
        String::new()
    }
}

#[derive(Debug, Clone)]
enum MemoryEntry {
    Hit {
        path: String,
        mtime_secs: u64,
        style_token: String,
    },
    /// Negative cache (session only; not written to disk).
    Miss,
}

fn icon_revision(mtime_secs: u64, style_token: &str) -> String {
    format!("{mtime_secs}:{style_token}")
}

/// Resolves icons with memory → disk → source tiering.
pub struct CachedAppIconResolver<S: AppIconSource> {
    source: S,
    cache_dir: PathBuf,
    memory: Mutex<HashMap<String, MemoryEntry>>,
    style: Arc<dyn IconStyleSource>,
}

impl<S: AppIconSource> CachedAppIconResolver<S> {
    pub fn new(source: S, cache_dir: PathBuf) -> Self {
        Self::with_style(source, cache_dir, Arc::new(EmptyIconStyle))
    }

    pub fn with_style(source: S, cache_dir: PathBuf, style: Arc<dyn IconStyleSource>) -> Self {
        Self {
            source,
            cache_dir,
            memory: Mutex::new(HashMap::new()),
            style,
        }
    }

    #[cfg(test)]
    fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl<S: AppIconSource> AppIconResolver for CachedAppIconResolver<S> {
    fn resolve(&self, bundle_id: &str) -> Option<ResolvedAppIcon> {
        if bundle_id.is_empty() {
            return None;
        }

        let style_token = self.style.token();

        // Trust in-memory hits for the process lifetime (style checked every call).
        if let Ok(memory) = self.memory.lock() {
            if let Some(entry) = memory.get(bundle_id) {
                match entry {
                    MemoryEntry::Hit {
                        path,
                        mtime_secs,
                        style_token: cached_style,
                    } if cached_style == &style_token && Path::new(path).is_file() => {
                        return Some(ResolvedAppIcon {
                            path: path.clone(),
                            revision: icon_revision(*mtime_secs, cached_style),
                        });
                    }
                    MemoryEntry::Miss => {
                        // Still miss unless the app becomes locatable below.
                    }
                    MemoryEntry::Hit { .. } => {
                        // Style changed or file gone — fall through.
                    }
                }
            }
        }

        let current_mtime = self.source.probe_mtime(bundle_id);

        if let Ok(memory) = self.memory.lock() {
            if let Some(MemoryEntry::Miss) = memory.get(bundle_id) {
                current_mtime?;
            }
        }

        if let Some(mtime_secs) = current_mtime {
            if let Some(path) = load_disk_path(&self.cache_dir, bundle_id, mtime_secs, &style_token)
            {
                store_memory_hit(
                    &self.memory,
                    bundle_id,
                    path.clone(),
                    mtime_secs,
                    style_token.clone(),
                );
                return Some(ResolvedAppIcon {
                    path,
                    revision: icon_revision(mtime_secs, &style_token),
                });
            }
        }

        match self.source.fetch(bundle_id) {
            Some(fetched) => {
                let _ = store_disk(&self.cache_dir, bundle_id, &fetched, &style_token);
                let path = png_path(&self.cache_dir, bundle_id)
                    .to_string_lossy()
                    .into_owned();
                store_memory_hit(
                    &self.memory,
                    bundle_id,
                    path.clone(),
                    fetched.mtime_secs,
                    style_token.clone(),
                );
                Some(ResolvedAppIcon {
                    path,
                    revision: icon_revision(fetched.mtime_secs, &style_token),
                })
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
    #[serde(rename = "styleToken", default)]
    style_token: String,
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

fn load_disk_path(
    cache_dir: &Path,
    bundle_id: &str,
    expected_mtime: u64,
    expected_style: &str,
) -> Option<String> {
    let meta_bytes = fs::read(meta_path(cache_dir, bundle_id)).ok()?;
    let meta: DiskMeta = serde_json::from_slice(&meta_bytes).ok()?;
    if meta.mtime_secs != expected_mtime || meta.style_token != expected_style {
        return None;
    }
    let path = png_path(cache_dir, bundle_id);
    let meta_len = fs::metadata(&path).ok()?.len();
    if meta_len == 0 {
        return None;
    }
    Some(path.to_string_lossy().into_owned())
}

fn store_disk(
    cache_dir: &Path,
    bundle_id: &str,
    fetched: &FetchedIcon,
    style_token: &str,
) -> std::io::Result<()> {
    fs::create_dir_all(cache_dir)?;
    let png_target = png_path(cache_dir, bundle_id);
    let meta_target = meta_path(cache_dir, bundle_id);
    atomic_write(&png_target, &fetched.png)?;
    let meta = DiskMeta {
        mtime_secs: fetched.mtime_secs,
        style_token: style_token.to_owned(),
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
    path: String,
    mtime_secs: u64,
    style_token: String,
) {
    if let Ok(mut guard) = memory.lock() {
        guard.insert(
            bundle_id.to_string(),
            MemoryEntry::Hit {
                path,
                mtime_secs,
                style_token,
            },
        );
    }
}

fn store_memory_miss(memory: &Mutex<HashMap<String, MemoryEntry>>, bundle_id: &str) {
    if let Ok(mut guard) = memory.lock() {
        guard.insert(bundle_id.to_string(), MemoryEntry::Miss);
    }
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
