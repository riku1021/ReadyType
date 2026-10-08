use super::*;

struct StubResolver;

impl AppDisplayNameResolver for StubResolver {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        if bundle_id == "com.google.Chrome" {
            Some("Google Chrome".to_owned())
        } else {
            None
        }
    }
}

#[test]
fn maps_each_bundle_id_and_keeps_unresolved_as_none() {
    let ids = vec!["com.google.Chrome".to_owned(), "com.unknown.app".to_owned()];

    let names = execute(&StubResolver, &ids);

    assert_eq!(
        names.get("com.google.Chrome"),
        Some(&Some("Google Chrome".to_owned()))
    );
    assert_eq!(names.get("com.unknown.app"), Some(&None));
}

#[test]
fn deduplicates_via_map_keys() {
    let ids = vec![
        "com.google.Chrome".to_owned(),
        "com.google.Chrome".to_owned(),
    ];

    let names = execute(&StubResolver, &ids);

    assert_eq!(names.len(), 1);
}
