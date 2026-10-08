use super::*;

struct StubResolver;

impl AppIconResolver for StubResolver {
    fn resolve(&self, bundle_id: &str) -> Option<String> {
        if bundle_id == "com.known.app" {
            Some("data:image/png;base64,AAAA".to_string())
        } else {
            None
        }
    }
}

#[test]
fn maps_each_bundle_id_and_keeps_unresolved_as_none() {
    let ids = vec!["com.known.app".to_string(), "com.unknown.app".to_string()];

    let icons = execute(&StubResolver, &ids);

    assert_eq!(
        icons.get("com.known.app"),
        Some(&Some("data:image/png;base64,AAAA".to_string()))
    );
    assert_eq!(icons.get("com.unknown.app"), Some(&None));
}

#[test]
fn deduplicates_via_map_keys() {
    let ids = vec!["com.known.app".to_string(), "com.known.app".to_string()];

    let icons = execute(&StubResolver, &ids);

    assert_eq!(icons.len(), 1);
}
