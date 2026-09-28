//! Quit the whole process (same as tray 「終了」). Window close only hides.

use tauri::AppHandle;

#[tauri::command]
pub fn quit_app(app: AppHandle) {
    app.exit(0);
}
