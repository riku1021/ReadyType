//! Resolve OS application icons and display names together.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::application::ports::{
    app_display_name::AppDisplayNameResolver, app_icon::AppIconResolver,
};
use crate::application::use_cases::app_presentations;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPresentationDto {
    pub icon: Option<String>,
    pub display_name: Option<String>,
}

/// Returns icon + Finder-style display name per bundle id (`None` fields when unresolved).
#[tauri::command]
pub fn get_app_presentations(
    icons: State<'_, Arc<dyn AppIconResolver>>,
    display_names: State<'_, Arc<dyn AppDisplayNameResolver>>,
    bundle_ids: Vec<String>,
) -> BTreeMap<String, AppPresentationDto> {
    app_presentations::execute(
        icons.inner().as_ref(),
        display_names.inner().as_ref(),
        &bundle_ids,
    )
    .into_iter()
    .map(|(bundle_id, presentation)| {
        (
            bundle_id,
            AppPresentationDto {
                icon: presentation.icon,
                display_name: presentation.display_name,
            },
        )
    })
    .collect()
}
