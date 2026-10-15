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
