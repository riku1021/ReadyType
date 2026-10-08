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
mod tests;
