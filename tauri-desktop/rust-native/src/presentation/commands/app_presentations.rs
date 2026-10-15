//! Resolve OS application icons and display names together.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::Serialize;
use tauri::State;

use crate::application::ports::{
    app_display_name::AppDisplayNameResolver, app_icon::AppIconResolver,
};
use crate::application::use_cases::app_presentations;

/// FE listens for this after System Settings changes Icon & widget style.
pub const ICON_STYLE_CHANGED_EVENT: &str = "app-presentations://icon-style-changed";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IconStyleChangedDto {
    pub style_token: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppPresentationDto {
    /// Absolute path to a cached PNG (load via asset protocol / `convertFileSrc`).
    pub icon: Option<String>,
    /// Cache-busting token for the asset URL (`mtimeSecs:styleToken`).
    pub icon_revision: Option<String>,
    pub display_name: Option<String>,
}

/// Returns cached icon path + Finder-style display name per bundle id (`None` when unresolved).
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
                icon_revision: presentation.icon_revision,
                display_name: presentation.display_name,
            },
        )
    })
    .collect()
}
