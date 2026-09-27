//! Resolve OS application icons for the given bundle ids.

use std::collections::BTreeMap;
use std::sync::Arc;

use tauri::State;

use crate::application::ports::app_icon::AppIconResolver;
use crate::application::use_cases::app_icons;

/// Returns a PNG data URL per bundle id (`None` when unresolved / unsupported).
#[tauri::command]
pub fn get_app_icons(
    resolver: State<'_, Arc<dyn AppIconResolver>>,
    bundle_ids: Vec<String>,
) -> BTreeMap<String, Option<String>> {
    app_icons::execute(resolver.inner().as_ref(), &bundle_ids)
}
