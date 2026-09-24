//! macOS window chrome helpers (titlebar / native background).

use tauri::{Runtime, WebviewWindow};

/// CSS `colors.background.main` と揃える（`react-ui/src/styles/tokens/color.ts`）。
const BACKGROUND_MAIN_LIGHT: (f64, f64, f64) = (0xEA as f64, 0xEA as f64, 0xEA as f64);
const BACKGROUND_MAIN_DARK: (f64, f64, f64) = (0x22 as f64, 0x22 as f64, 0x22 as f64);

/// Transparent タイトルバー越しに見える NSWindow 背景をアプリの `background.main` に合わせる。
pub fn set_window_background_main<R: Runtime>(window: &WebviewWindow<R>, is_dark: bool) {
    use objc2_app_kit::{NSColor, NSWindow};

    let (r, g, b) = if is_dark {
        BACKGROUND_MAIN_DARK
    } else {
        BACKGROUND_MAIN_LIGHT
    };

    let Ok(ns_window_ptr) = window.ns_window() else {
        return;
    };
    let ns_window_ptr = ns_window_ptr as *const NSWindow;
    // SAFETY: Tauri が返す ns_window は有効な NSWindow ポインタ。
    let ns_window = unsafe { &*ns_window_ptr };
    let bg_color = NSColor::colorWithRed_green_blue_alpha(r / 255.0, g / 255.0, b / 255.0, 1.0);
    ns_window.setBackgroundColor(Some(&bg_color));
}
