//! Resolve icons for a set of application bundle ids.

use std::collections::BTreeMap;

use crate::application::ports::app_icon::{AppIconResolver, ResolvedAppIcon};

/// Resolves an icon for each requested bundle id, preserving unresolved entries as `None`.
pub fn execute(
    resolver: &dyn AppIconResolver,
    bundle_ids: &[String],
) -> BTreeMap<String, Option<ResolvedAppIcon>> {
    bundle_ids
        .iter()
        .map(|bundle_id| (bundle_id.clone(), resolver.resolve(bundle_id)))
        .collect()
}

#[cfg(test)]
mod tests;
