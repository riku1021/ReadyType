//! Resolve display names for a set of application bundle ids.

use std::collections::BTreeMap;

use crate::application::ports::app_display_name::AppDisplayNameResolver;

/// Resolves a display name for each requested bundle id, preserving unresolved entries as `None`.
pub fn execute(
    resolver: &dyn AppDisplayNameResolver,
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
}
