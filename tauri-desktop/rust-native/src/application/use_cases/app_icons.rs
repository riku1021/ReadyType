//! Resolve icons for a set of application bundle ids.

use std::collections::BTreeMap;

use crate::application::ports::app_icon::AppIconResolver;

/// Resolves an icon for each requested bundle id, preserving unresolved entries as `None`.
pub fn execute(
    resolver: &dyn AppIconResolver,
    bundle_ids: &[String],
) -> BTreeMap<String, Option<String>> {
    bundle_ids
        .iter()
        .map(|bundle_id| (bundle_id.clone(), resolver.resolve(bundle_id)))
        .collect()
}

#[cfg(test)]
mod tests {
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
}
