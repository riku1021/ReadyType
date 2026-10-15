use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};

struct StubSource {
    png: Vec<u8>,
    mtime: Mutex<u64>,
    fetch_count: AtomicUsize,
    probe_count: AtomicUsize,
    missing: Mutex<bool>,
}

impl StubSource {
    fn new(mtime: u64) -> Self {
        Self {
            png: b"PNGDATA".to_vec(),
            mtime: Mutex::new(mtime),
            fetch_count: AtomicUsize::new(0),
            probe_count: AtomicUsize::new(0),
            missing: Mutex::new(false),
        }
    }
}

impl AppIconSource for StubSource {
    fn probe_mtime(&self, _bundle_id: &str) -> Option<u64> {
        self.probe_count.fetch_add(1, Ordering::SeqCst);
        if *self.missing.lock().expect("lock") {
            return None;
        }
        Some(*self.mtime.lock().expect("lock"))
    }

    fn fetch(&self, _bundle_id: &str) -> Option<FetchedIcon> {
        self.fetch_count.fetch_add(1, Ordering::SeqCst);
        if *self.missing.lock().expect("lock") {
            return None;
        }
        Some(FetchedIcon {
            png: self.png.clone(),
            mtime_secs: *self.mtime.lock().expect("lock"),
        })
    }
}

struct MutableStyle {
    token: Mutex<String>,
}

impl IconStyleSource for MutableStyle {
    fn token(&self) -> String {
        self.token.lock().expect("lock").clone()
    }
}

fn temp_cache_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "readytype-icon-cache-{label}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp cache dir");
    dir
}

#[test]
fn memory_hit_skips_second_fetch_and_probe() {
    let dir = temp_cache_dir("memory");
    let source = StubSource::new(10);
    let resolver = CachedAppIconResolver::new(source, dir.clone());

    let first = resolver.resolve("com.example.App").expect("first");
    let second = resolver.resolve("com.example.App").expect("second");
    assert_eq!(first, second);
    assert!(Path::new(&first.path).is_file());
    assert_eq!(first.revision, "10:");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);
    assert_eq!(resolver.source.probe_count.load(Ordering::SeqCst), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn disk_hit_after_memory_cleared() {
    let dir = temp_cache_dir("disk");
    let source = StubSource::new(20);
    let resolver = CachedAppIconResolver::new(source, dir.clone());
    let first = resolver.resolve("com.example.App").expect("first");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);

    // Drop memory by building a new resolver on the same disk.
    let source2 = StubSource::new(20);
    let resolver2 = CachedAppIconResolver::new(source2, dir.clone());
    let second = resolver2.resolve("com.example.App").expect("disk");
    assert_eq!(first.path, second.path);
    assert_eq!(first.revision, second.revision);
    assert_eq!(resolver2.source.fetch_count.load(Ordering::SeqCst), 0);
    assert!(png_path(&dir, "com.example.App").exists());
    assert!(meta_path(&dir, "com.example.App").exists());

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn mtime_bump_misses_disk_and_refetches() {
    let dir = temp_cache_dir("stale");
    let source = StubSource::new(30);
    let resolver = CachedAppIconResolver::new(source, dir.clone());
    let _ = resolver.resolve("com.example.App").expect("first");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);

    *resolver.source.mtime.lock().expect("lock") = 31;
    // Clear memory so disk path is exercised with stale token.
    resolver.memory.lock().expect("lock").clear();

    let _ = resolver.resolve("com.example.App").expect("refetch");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 2);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn style_token_change_refetches() {
    let dir = temp_cache_dir("style");
    let style = Arc::new(MutableStyle {
        token: Mutex::new("default".into()),
    });
    let source = StubSource::new(35);
    let resolver = CachedAppIconResolver::with_style(
        source,
        dir.clone(),
        Arc::clone(&style) as Arc<dyn IconStyleSource>,
    );

    let first = resolver.resolve("com.example.App").expect("first");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);
    assert_eq!(first.revision, "35:default");

    *style.token.lock().expect("lock") = "tinted".into();
    let second = resolver.resolve("com.example.App").expect("style refetch");
    assert_eq!(first.path, second.path);
    assert_eq!(second.revision, "35:tinted");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 2);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn corrupt_sidecar_is_a_miss() {
    let dir = temp_cache_dir("corrupt");
    fs::write(png_path(&dir, "com.example.App"), b"PNGDATA").expect("png");
    fs::write(meta_path(&dir, "com.example.App"), b"{not-json").expect("meta");

    let source = StubSource::new(40);
    let resolver = CachedAppIconResolver::new(source, dir.clone());
    let _ = resolver.resolve("com.example.App").expect("fetch");
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn atomic_write_leaves_no_tmp() {
    let dir = temp_cache_dir("atomic");
    let source = StubSource::new(50);
    let resolver = CachedAppIconResolver::new(source, dir.clone());
    let _ = resolver.resolve("com.example.App").expect("write");

    let leftovers: Vec<_> = fs::read_dir(&dir)
        .expect("read")
        .filter_map(Result::ok)
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp"))
        .collect();
    assert!(leftovers.is_empty(), "tmp leftovers: {leftovers:?}");

    let _ = fs::remove_dir_all(dir);
}

#[test]
fn negative_miss_is_memory_only() {
    let dir = temp_cache_dir("negative");
    let source = StubSource::new(60);
    *source.missing.lock().expect("lock") = true;
    let resolver = CachedAppIconResolver::new(source, dir.clone());

    assert!(resolver.resolve("com.missing.App").is_none());
    assert!(resolver.resolve("com.missing.App").is_none());
    assert_eq!(resolver.source.fetch_count.load(Ordering::SeqCst), 1);
    // Second resolve still probes once to detect an app that appears later.
    assert_eq!(resolver.source.probe_count.load(Ordering::SeqCst), 2);
    assert!(!png_path(resolver.cache_dir(), "com.missing.App").exists());

    let _ = fs::remove_dir_all(dir);
}
