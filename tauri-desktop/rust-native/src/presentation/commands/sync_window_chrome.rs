//! Sync native window chrome with the resolved light/dark theme.

use tauri::{AppHandle, Manager, Runtime};

#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WindowChromeTheme {
    Light,
    Dark,
}

/// Transparent タイトルバーの NSWindow 背景を `background.main` に同期する。
#[tauri::command]
pub fn sync_window_chrome_theme<R: Runtime>(
    app: AppHandle<R>,
    theme: WindowChromeTheme,
) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let window = app
            .get_webview_window("main")
            .ok_or_else(|| "main window not found".to_string())?;
        let is_dark = matches!(theme, WindowChromeTheme::Dark);
        crate::infrastructure::platform::macos::set_window_background_main(&window, is_dark);
        Ok(())
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (app, theme);
        Ok(())
    }
}
