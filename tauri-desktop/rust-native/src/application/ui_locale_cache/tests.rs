use super::UiLocaleCache;

#[test]
fn stores_and_loads_resolved_locales() {
    let cache = UiLocaleCache::new("ja");
    assert_eq!(cache.load(), "ja");
    cache.store("en");
    assert_eq!(cache.load(), "en");
    cache.store("zh");
    assert_eq!(cache.load(), "zh");
    cache.store("system");
    assert_eq!(cache.load(), "ja");
}
