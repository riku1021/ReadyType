//! Quick Set Level-1 popover window (hotkey-driven).

use std::sync::Arc;

use tauri::{
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, WebviewUrl, WebviewWindowBuilder,
};

use crate::application::{quick_set::QuickSetSession, switcher_controller::SwitcherController};
use crate::infrastructure::desktop::main_window;
use crate::presentation::input_source_dto::{QuickSetOpenEventDto, QuickSetSessionDto};

pub const QUICK_SET_WINDOW_LABEL: &str = "quick-set";
pub const QUICK_SET_EVENT: &str = "readytype://quick-set";

const WINDOW_WIDTH: f64 = 320.0;
/// ヘッダー／ルール名／選択肢ちょうど2行／編集＋削除フッターが収まる高さ
const WINDOW_HEIGHT: f64 = 448.0;
const GAP: f64 = 8.0;

pub fn ensure_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(QUICK_SET_WINDOW_LABEL).is_some() {
        return Ok(());
    }

    let window = WebviewWindowBuilder::new(
        app,
        QUICK_SET_WINDOW_LABEL,
        WebviewUrl::App("index.html".into()),
    )
    .title("ReadyType")
    .inner_size(WINDOW_WIDTH, WINDOW_HEIGHT)
    .resizable(false)
    .maximizable(false)
    .minimizable(false)
    .closable(true)
    .decorations(false)
    .transparent(true)
    .always_on_top(true)
    .visible(false)
    .skip_taskbar(true)
    .build()
    .map_err(|error| error.to_string())?;

    #[cfg(target_os = "macos")]
    {
        crate::infrastructure::platform::macos::set_window_background_transparent(&window);
    }

    Ok(())
}

pub fn open(app: &AppHandle) -> Result<(), String> {
    let controller = app
        .try_state::<Arc<SwitcherController>>()
        .ok_or_else(|| "switcher controller unavailable".to_owned())?;

    // Capture focus BEFORE showing ReadyType UI. On failure still open the window with an
    // empty state (Raycast-style feedback) — never apply a stale foreign snapshot.
    let prepared = controller.prepare_quick_set_session();
    let event = match &prepared {
        Ok(session) => QuickSetOpenEventDto::Ready {
            session: Box::new(QuickSetSessionDto::from(session)),
        },
        Err(error) => {
            eprintln!("quick-set session unavailable: {error}");
            let _ = controller.clear_quick_set_session();
            QuickSetOpenEventDto::Unavailable
        }
    };

    ensure_window(app)?;
    let window = app
        .get_webview_window(QUICK_SET_WINDOW_LABEL)
        .ok_or_else(|| "quick-set window missing".to_owned())?;

    match &prepared {
        Ok(session) => position_near_focus(app, &window, session)?,
        Err(_) => position_centered(app, &window)?,
    }

    // Keep Accessory while Quick Set is open so the Dock stays hidden in BG residency.
    // Main UI still switches to Regular via `show_main_window` / `show_main_for_app`.

    // Webview-targeted emit; FE listens via getCurrentWebviewWindow().listen.
    let _ = app.emit_to(QUICK_SET_WINDOW_LABEL, QUICK_SET_EVENT, &event);
    let _ = window.show();
    let _ = window.set_focus();
    Ok(())
}

pub fn hide(app: &AppHandle) {
    if let Some(window) = app.get_webview_window(QUICK_SET_WINDOW_LABEL) {
        let _ = window.hide();
    }
    if let Some(controller) = app.try_state::<Arc<SwitcherController>>() {
        let _ = controller.clear_quick_set_session();
    }
}

pub fn show_main_for_app(app: &AppHandle, app_id: &str) {
    hide(app);
    let route = format!("/apps/{}/", urlencoding_lightweight(app_id));
    main_window::show_main_window(app, Some(&route));
}

fn position_near_focus(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    session: &QuickSetSession,
) -> Result<(), String> {
    let frame = {
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::platform::macos::focused_element_frame(session.pid)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = session;
            None
        }
    };

    let (x, y) = if let Some(frame) = frame {
        let mut x = frame.x;
        let mut y = frame.y + frame.height + GAP;
        // Prefer aligning to the left of the field; clamp later via monitor if available.
        if let Some(monitor) = app.primary_monitor().ok().flatten() {
            let scale = monitor.scale_factor();
            let size = monitor.size();
            let position = monitor.position();
            let screen_x = f64::from(position.x) / scale;
            let screen_y = f64::from(position.y) / scale;
            let screen_w = f64::from(size.width) / scale;
            let screen_h = f64::from(size.height) / scale;
            if y + WINDOW_HEIGHT > screen_y + screen_h {
                y = frame.y - WINDOW_HEIGHT - GAP;
            }
            if x + WINDOW_WIDTH > screen_x + screen_w {
                x = screen_x + screen_w - WINDOW_WIDTH - GAP;
            }
            if x < screen_x {
                x = screen_x + GAP;
            }
            if y < screen_y {
                y = screen_y + GAP;
            }
        }
        (x, y)
    } else {
        return position_centered(app, window);
    };

    let _ = window.set_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let _ = window.set_position(LogicalPosition::new(x, y));
    Ok(())
}

fn position_centered(app: &AppHandle, window: &tauri::WebviewWindow) -> Result<(), String> {
    let (x, y) = if let Some(monitor) = app.primary_monitor().ok().flatten() {
        let scale = monitor.scale_factor();
        let size = monitor.size();
        let position = monitor.position();
        let screen_x = f64::from(position.x) / scale;
        let screen_y = f64::from(position.y) / scale;
        let screen_w = f64::from(size.width) / scale;
        let screen_h = f64::from(size.height) / scale;
        (
            screen_x + (screen_w - WINDOW_WIDTH) / 2.0,
            screen_y + (screen_h - WINDOW_HEIGHT) / 2.0,
        )
    } else {
        (200.0, 200.0)
    };

    let _ = window.set_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
    let _ = window.set_position(LogicalPosition::new(x, y));
    Ok(())
}

fn urlencoding_lightweight(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(ch),
            _ => {
                for byte in ch.to_string().as_bytes() {
                    out.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    out
}
