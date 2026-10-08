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
mod tests;
