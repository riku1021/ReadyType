//! Resolve icon + display name for a set of application bundle ids in one pass.

use std::collections::BTreeMap;

use crate::application::ports::{
    app_display_name::AppDisplayNameResolver, app_icon::AppIconResolver,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPresentation {
    pub icon: Option<String>,
    pub display_name: Option<String>,
}

/// Resolves icon and display name for each bundle id (`None` when unresolved).
pub fn execute(
    icons: &dyn AppIconResolver,
    display_names: &dyn AppDisplayNameResolver,
    bundle_ids: &[String],
) -> BTreeMap<String, AppPresentation> {
    bundle_ids
        .iter()
        .map(|bundle_id| {
            (
                bundle_id.clone(),
                AppPresentation {
                    icon: icons.resolve(bundle_id),
                    display_name: display_names.resolve(bundle_id),
                },
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubIcons;
    struct StubNames;

    impl AppIconResolver for StubIcons {
        fn resolve(&self, bundle_id: &str) -> Option<String> {
            (bundle_id == "com.known.app").then(|| "data:image/png;base64,AAAA".to_owned())
        }
    }

    impl AppDisplayNameResolver for StubNames {
        fn resolve(&self, bundle_id: &str) -> Option<String> {
            (bundle_id == "com.known.app").then(|| "Known".to_owned())
        }
    }

    #[test]
    fn maps_icon_and_display_name_together() {
        let ids = vec!["com.known.app".to_owned(), "com.unknown.app".to_owned()];
        let result = execute(&StubIcons, &StubNames, &ids);

        assert_eq!(
            result.get("com.known.app"),
            Some(&AppPresentation {
                icon: Some("data:image/png;base64,AAAA".to_owned()),
                display_name: Some("Known".to_owned()),
            })
        );
        assert_eq!(
            result.get("com.unknown.app"),
            Some(&AppPresentation {
                icon: None,
                display_name: None,
            })
        );
    }
}
