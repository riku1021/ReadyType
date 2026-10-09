//! Quick Set グローバルショートカットの登録・差し替え。

use tauri::AppHandle;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::infrastructure::desktop::quick_set_window;

fn open_handler(
    handle: AppHandle,
) -> impl Fn(&AppHandle, &Shortcut, tauri_plugin_global_shortcut::ShortcutEvent) + Send + Sync + 'static
{
    move |_app, _shortcut, event| {
        if event.state != ShortcutState::Pressed {
            return;
        }
        if let Err(error) = quick_set_window::open(&handle) {
            eprintln!("quick-set open failed: {error}");
        }
    }
}

/// 起動時などに accelerator を登録する。
pub fn register(app: &AppHandle, accelerator: &str) -> Result<(), String> {
    let shortcut = accelerator
        .parse::<Shortcut>()
        .map_err(|error| format!("invalid quick-set shortcut: {error}"))?;
    let handle = app.clone();
    app.global_shortcut()
        .on_shortcut(shortcut, open_handler(handle))
        .map_err(|error| format!("failed to register quick-set shortcut: {error}"))
}

/// OS に当該 accelerator が登録済みか（パース不能なら false）。
pub fn is_registered(app: &AppHandle, accelerator: &str) -> bool {
    let Ok(shortcut) = accelerator.parse::<Shortcut>() else {
        return false;
    };
    app.global_shortcut().is_registered(shortcut)
}

/// 旧 accelerator を外し、新しいものを登録する。新登録失敗時は旧を復元する。
pub fn replace(app: &AppHandle, previous: &str, next: &str) -> Result<(), String> {
    if previous == next {
        return Ok(());
    }

    let previous_shortcut = previous
        .parse::<Shortcut>()
        .map_err(|error| format!("invalid previous quick-set shortcut: {error}"))?;

    let _ = app.global_shortcut().unregister(previous_shortcut);
    match register(app, next) {
        Ok(()) => Ok(()),
        Err(error) => {
            let _ = register(app, previous);
            Err(error)
        }
    }
}
