use std::sync::Arc;

use serde::Deserialize;
use tauri::State;

use crate::infrastructure::desktop::app_update::{
    AppUpdateClientAck, AppUpdateRuntime, AppUpdateSnapshot, LegacyUpdatePreferences,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyUpdatePreferencesDto {
    pub auto_check: Option<bool>,
    pub auto_download: Option<bool>,
    pub last_checked_at_unix_ms: Option<u64>,
}

impl From<LegacyUpdatePreferencesDto> for LegacyUpdatePreferences {
    fn from(value: LegacyUpdatePreferencesDto) -> Self {
        Self {
            auto_check: value.auto_check,
            auto_download: value.auto_download,
            last_checked_at_unix_ms: value.last_checked_at_unix_ms,
        }
    }
}

fn shared(state: &State<'_, Arc<AppUpdateRuntime>>) -> Arc<AppUpdateRuntime> {
    Arc::clone(state.inner())
}

/// 画面起動時に旧 localStorage を一度だけ取り込み、スケジューラを進める。
#[tauri::command]
pub async fn ack_app_update_client(
    runtime: State<'_, Arc<AppUpdateRuntime>>,
    legacy: Option<LegacyUpdatePreferencesDto>,
) -> Result<AppUpdateClientAck, String> {
    Ok(shared(&runtime).accept_client(legacy.map(Into::into)).await)
}

#[tauri::command]
pub async fn set_app_update_auto_check(
    runtime: State<'_, Arc<AppUpdateRuntime>>,
    enabled: bool,
) -> Result<AppUpdateSnapshot, String> {
    Ok(shared(&runtime).set_auto_check(enabled).await)
}

#[tauri::command]
pub async fn set_app_update_auto_download(
    runtime: State<'_, Arc<AppUpdateRuntime>>,
    enabled: bool,
) -> Result<AppUpdateSnapshot, String> {
    Ok(shared(&runtime).set_auto_download(enabled).await)
}

#[tauri::command]
pub async fn check_app_update_now(
    runtime: State<'_, Arc<AppUpdateRuntime>>,
) -> Result<AppUpdateSnapshot, String> {
    Ok(shared(&runtime).check_now().await)
}

/// ダウンロード済みならインストールして再起動する。窓は前面に出さない。
#[tauri::command]
pub async fn install_app_update(
    runtime: State<'_, Arc<AppUpdateRuntime>>,
) -> Result<AppUpdateSnapshot, String> {
    Ok(shared(&runtime).install().await)
}
