use super::*;
use crate::application::ports::app_icon::ResolvedAppIcon;

struct StubResolver;

impl AppIconResolver for StubResolver {
    fn resolve(&self, bundle_id: &str) -> Option<ResolvedAppIcon> {
        if bundle_id == "com.known.app" {
            Some(ResolvedAppIcon {
                path: "/tmp/com.known.app.png".to_string(),
                revision: "1:".to_string(),
            })
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
        Some(&Some(ResolvedAppIcon {
            path: "/tmp/com.known.app.png".to_string(),
            revision: "1:".to_string(),
        }))
    );
    assert_eq!(icons.get("com.unknown.app"), Some(&None));
}

#[test]
fn deduplicates_via_map_keys() {
    let ids = vec!["com.known.app".to_string(), "com.known.app".to_string()];

    let icons = execute(&StubResolver, &ids);

    assert_eq!(icons.len(), 1);
}
