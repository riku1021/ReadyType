//! Lab 切替テスト用ネイティブ入力窓のデスクトップ入口。

use std::sync::Arc;

/// Lab 欄フォーカス時に呼ばれる（Identifier 文字列）。
pub type LabFocusHandler = Arc<dyn Fn(&str) + Send + Sync>;

/// Lab 切替テスト窓を開く（macOS のみ）。
pub fn show(on_focus: LabFocusHandler) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        crate::infrastructure::platform::macos::lab_switch_window::show(on_focus)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = on_focus;
        Err("lab switch window is only available on macOS".to_owned())
    }
}
