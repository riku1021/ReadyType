use std::sync::atomic::{AtomicUsize, Ordering};

use super::*;

struct CountingResolver {
    hits: AtomicUsize,
}

impl AppDisplayNameResolver for CountingResolver {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        self.hits.fetch_add(1, Ordering::SeqCst);
        (bundle_id == "com.known.app").then(|| "Known".to_owned())
    }
}

struct DynamicResolver {
    hits: AtomicUsize,
}

impl AppDisplayNameResolver for DynamicResolver {
    fn resolve(&self, app_id: &str) -> Option<String> {
        let hit = self.hits.fetch_add(1, Ordering::SeqCst);
        (app_id == "editor.exe").then(|| format!("Editor {hit}"))
    }

    fn is_bundle_resolution_cacheable(&self) -> bool {
        false
    }
}

#[test]
fn second_resolve_uses_memory() {
    let resolver = CachedAppDisplayNameResolver::new(CountingResolver {
        hits: AtomicUsize::new(0),
    });

    assert_eq!(resolver.resolve("com.known.app").as_deref(), Some("Known"));
    assert_eq!(resolver.resolve("com.known.app").as_deref(), Some("Known"));
    assert_eq!(resolver.inner.hits.load(Ordering::SeqCst), 1);
}

#[test]
fn negative_miss_is_cached() {
    let resolver = CachedAppDisplayNameResolver::new(CountingResolver {
        hits: AtomicUsize::new(0),
    });

    assert!(resolver.resolve("com.missing.app").is_none());
    assert!(resolver.resolve("com.missing.app").is_none());
    assert_eq!(resolver.inner.hits.load(Ordering::SeqCst), 1);
}

#[test]
fn dynamic_bundle_resolutions_are_refreshed() {
    let resolver = CachedAppDisplayNameResolver::new(DynamicResolver {
        hits: AtomicUsize::new(0),
    });

    assert_eq!(resolver.resolve("editor.exe").as_deref(), Some("Editor 0"));
    assert_eq!(resolver.resolve("editor.exe").as_deref(), Some("Editor 1"));
    assert_eq!(resolver.inner.hits.load(Ordering::SeqCst), 2);
}
