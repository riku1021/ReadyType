//! Quick Set Level-1 popover window (hotkey-driven).
//!
//! On macOS the webview is converted to an `NSPanel` so it can appear on a
//! fullscreen Space (`FullScreenAuxiliary`) without kicking the frontmost app
//! out of fullscreen (non-activating + `orderFrontRegardless`).

use std::sync::Arc;

use tauri::{AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, Monitor};

#[cfg(not(target_os = "macos"))]
use tauri::{WebviewUrl, WebviewWindowBuilder};

use crate::application::{quick_set::QuickSetSession, switcher_controller::SwitcherController};
use crate::infrastructure::desktop::main_window;
use crate::presentation::input_source_dto::{QuickSetOpenEventDto, QuickSetSessionDto};

pub const QUICK_SET_WINDOW_LABEL: &str = "quick-set";
pub const QUICK_SET_EVENT: &str = "readytype://quick-set";

const WINDOW_WIDTH: f64 = 320.0;
/// ヘッダー／ルール名／選択肢ちょうど2行／編集＋削除フッターが収まる高さ
const WINDOW_HEIGHT: f64 = 448.0;
/// 削除行（SettingItem: padding 16px×2 + icon 24px）とその上の区切り 1px。
/// 「このルールを削除」が無いときはこの分だけ窓を縮める。
const DELETE_RULE_ROW_HEIGHT: f64 = 57.0;
const GAP: f64 = 8.0;

fn quick_set_window_height(shows_delete_rule: bool) -> f64 {
    if shows_delete_rule {
        WINDOW_HEIGHT
    } else {
        WINDOW_HEIGHT - DELETE_RULE_ROW_HEIGHT
    }
}

#[cfg(target_os = "macos")]
mod panel_macos {
    use tauri::{AppHandle, LogicalSize, Manager, WebviewUrl};
    use tauri_nspanel::{
        tauri_panel, CollectionBehavior, ManagerExt, PanelBuilder, PanelLevel, StyleMask,
    };

    use super::{QUICK_SET_WINDOW_LABEL, WINDOW_HEIGHT, WINDOW_WIDTH};

    tauri_panel! {
        panel!(QuickSetPanel {
            config: {
                can_become_key_window: true,
                can_become_main_window: false,
                is_floating_panel: true
            }
        })
    }

    pub(super) fn ensure_panel(app: &AppHandle) -> Result<(), String> {
        PanelBuilder::<_, QuickSetPanel>::new(app, QUICK_SET_WINDOW_LABEL)
            .url(WebviewUrl::App("index.html".into()))
            .title("ReadyType")
            .size(tauri::Size::Logical(LogicalSize::new(
                WINDOW_WIDTH,
                WINDOW_HEIGHT,
            )))
            .with_window(|window| {
                window
                    .resizable(false)
                    .maximizable(false)
                    .minimizable(false)
                    .closable(true)
                    .decorations(false)
                    .transparent(true)
                    .visible(false)
                    .skip_taskbar(true)
            })
            // Creation must not flip Accessory → Regular (Dock flash / leave fullscreen).
            .no_activate(true)
            .floating(true)
            .level(PanelLevel::Floating)
            .hides_on_deactivate(false)
            .transparent(true)
            // Ephemeral overlay: follow the active Space (incl. fullscreen) without lingering
            // on every Space the way CanJoinAllSpaces + Stationary would.
            .collection_behavior(
                CollectionBehavior::new()
                    .full_screen_auxiliary()
                    .move_to_active_space()
                    .ignores_cycle(),
            )
            .add_style_mask(StyleMask::empty().nonactivating_panel())
            .build()
            .map_err(|error| error.to_string())?;

        if let Some(window) = app.get_webview_window(QUICK_SET_WINDOW_LABEL) {
            crate::infrastructure::platform::macos::set_window_background_transparent(&window);
        }

        Ok(())
    }

    pub(super) fn show(app: &AppHandle) -> Result<(), String> {
        let panel = app
            .get_webview_panel(QUICK_SET_WINDOW_LABEL)
            .map_err(|error| format!("quick-set panel missing: {error:?}"))?;
        // orderFrontRegardless + makeKeyWindow — does not activate the app like set_focus().
        panel.show_and_make_key();
        Ok(())
    }

    pub(super) fn hide(app: &AppHandle) {
        if let Ok(panel) = app.get_webview_panel(QUICK_SET_WINDOW_LABEL) {
            panel.hide();
        }
    }
}

pub fn ensure_window(app: &AppHandle) -> Result<(), String> {
    if app.get_webview_window(QUICK_SET_WINDOW_LABEL).is_some() {
        return Ok(());
    }

    #[cfg(target_os = "macos")]
    {
        panel_macos::ensure_panel(app)
    }

    #[cfg(not(target_os = "macos"))]
    {
        WebviewWindowBuilder::new(
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
        .map(|_| ())
        .map_err(|error| error.to_string())
    }
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

    let shows_delete_rule = matches!(&prepared, Ok(session) if session.matched_rule_id.is_some());
    let height = quick_set_window_height(shows_delete_rule);
    match &prepared {
        Ok(session) => position_near_focus(app, &window, session, height)?,
        Err(_) => position_centered_near_cursor(app, &window, height)?,
    }

    // Keep Accessory while Quick Set is open so the Dock stays hidden in BG residency.
    // Main UI still switches to Regular via `show_main_window` / `show_main_for_app`.

    // Webview-targeted emit; FE listens via getCurrentWebviewWindow().listen.
    let _ = app.emit_to(QUICK_SET_WINDOW_LABEL, QUICK_SET_EVENT, &event);

    #[cfg(target_os = "macos")]
    {
        panel_macos::show(app)?;
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.show();
        let _ = window.set_focus();
    }
    Ok(())
}

pub fn hide(app: &AppHandle) {
    #[cfg(target_os = "macos")]
    {
        panel_macos::hide(app);
    }
    #[cfg(not(target_os = "macos"))]
    {
        if let Some(window) = app.get_webview_window(QUICK_SET_WINDOW_LABEL) {
            let _ = window.hide();
        }
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

#[derive(Debug, Clone, Copy, PartialEq)]
struct LogicalRect {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

impl LogicalRect {
    fn contains(&self, x: f64, y: f64) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

fn monitor_logical_rect(monitor: &Monitor) -> LogicalRect {
    let scale = monitor.scale_factor();
    let position = monitor.position();
    let size = monitor.size();
    LogicalRect {
        x: f64::from(position.x) / scale,
        y: f64::from(position.y) / scale,
        width: f64::from(size.width) / scale,
        height: f64::from(size.height) / scale,
    }
}

fn primary_monitor_rect(app: &AppHandle) -> Option<LogicalRect> {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|monitor| monitor_logical_rect(&monitor))
}

fn monitor_contains_physical(monitor: &Monitor, x: f64, y: f64) -> bool {
    let position = monitor.position();
    let size = monitor.size();
    let left = f64::from(position.x);
    let top = f64::from(position.y);
    let right = left + f64::from(size.width);
    let bottom = top + f64::from(size.height);
    x >= left && x < right && y >= top && y < bottom
}

fn monitor_rect_for_logical_point(app: &AppHandle, x: f64, y: f64) -> Option<LogicalRect> {
    app.available_monitors()
        .unwrap_or_default()
        .into_iter()
        .map(|monitor| monitor_logical_rect(&monitor))
        .find(|rect| rect.contains(x, y))
        .or_else(|| primary_monitor_rect(app))
}

fn monitor_rect_for_physical_point(app: &AppHandle, x: f64, y: f64) -> Option<LogicalRect> {
    app.available_monitors()
        .unwrap_or_default()
        .into_iter()
        .find(|monitor| monitor_contains_physical(monitor, x, y))
        .map(|monitor| monitor_logical_rect(&monitor))
        .or_else(|| primary_monitor_rect(app))
}

/// Place the popover near an AX frame, flipped above when it would overflow the bottom.
fn place_near_anchor(anchor: LogicalRect, monitor: LogicalRect, window_height: f64) -> (f64, f64) {
    let mut x = anchor.x;
    let mut y = anchor.y + anchor.height + GAP;
    if y + window_height > monitor.y + monitor.height {
        y = anchor.y - window_height - GAP;
    }
    if x + WINDOW_WIDTH > monitor.x + monitor.width {
        x = monitor.x + monitor.width - WINDOW_WIDTH - GAP;
    }
    if x < monitor.x {
        x = monitor.x + GAP;
    }
    if y < monitor.y {
        y = monitor.y + GAP;
    }
    if y + window_height > monitor.y + monitor.height {
        y = (monitor.y + monitor.height - window_height - GAP).max(monitor.y + GAP);
    }
    (x, y)
}

fn center_on_monitor(monitor: LogicalRect, window_height: f64) -> (f64, f64) {
    (
        monitor.x + (monitor.width - WINDOW_WIDTH) / 2.0,
        monitor.y + (monitor.height - window_height) / 2.0,
    )
}

fn apply_window_position(window: &tauri::WebviewWindow, x: f64, y: f64, window_height: f64) {
    let _ = window.set_size(LogicalSize::new(WINDOW_WIDTH, window_height));
    let _ = window.set_position(LogicalPosition::new(x, y));
}

fn position_near_focus(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    session: &QuickSetSession,
    window_height: f64,
) -> Result<(), String> {
    let frame = {
        #[cfg(target_os = "macos")]
        {
            crate::infrastructure::platform::macos::focused_element_frame(session.pid)
        }
        #[cfg(not(target_os = "macos"))]
        {
            // Windows / Linux は入力欄の矩形をまだ取れない。正式対応するときは
            // この None を、フォーカス中の入力欄の Some(LogicalRect { x, y, width, height })
            // に差し替える。macOS の ElementFrame は別モジュールにあるため、ここでは使わない。
            // 型を省いた None は E0282 になり、Windows / Linux のビルドが通らない。
            let _ = session;
            None::<LogicalRect>
        }
    };

    let Some(frame) = frame else {
        return position_centered_near_cursor(app, window, window_height);
    };

    let anchor = LogicalRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    };
    let probe_x = frame.x + frame.width / 2.0;
    let probe_y = frame.y + frame.height / 2.0;
    let monitor = monitor_rect_for_logical_point(app, probe_x, probe_y)
        .ok_or_else(|| "no monitor available for quick-set placement".to_owned())?;
    let (x, y) = place_near_anchor(anchor, monitor, window_height);
    apply_window_position(window, x, y, window_height);
    Ok(())
}

fn position_centered_near_cursor(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    window_height: f64,
) -> Result<(), String> {
    let monitor = match app.cursor_position() {
        Ok(cursor) => monitor_rect_for_physical_point(app, cursor.x, cursor.y),
        Err(_) => primary_monitor_rect(app),
    };

    let (x, y) = match monitor {
        Some(rect) => center_on_monitor(rect, window_height),
        None => (200.0, 200.0),
    };
    apply_window_position(window, x, y, window_height);
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

#[cfg(test)]
mod tests;
