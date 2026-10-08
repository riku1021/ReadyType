//! Lab 切替テスト用のネイティブ入力窓。
//!
//! ReadyType 同一プロセスでは AXFocusedUIElement が空のまま残ることがあり
//!（WKWebView 由来）、本番モニタが Lab 欄を拾えない。そのため Lab 窓が key の間は
//! AppKit firstResponder（と可能なら AX）から Identifier を読み、切替ワーカーへ渡す。

use std::cell::RefCell;
use std::ffi::c_void;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, MainThreadOnly};
use objc2_app_kit::{
    NSAccessibility, NSApplication, NSBackingStoreType, NSControl, NSResponder, NSTextField,
    NSView, NSWindow, NSWindowStyleMask,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};

use crate::domain::input_source::{AxAttribute, LAB_SWITCH_FIELD_IDS};
use crate::infrastructure::platform::focus_monitor::AttributePlan;
use crate::infrastructure::platform::macos::accessibility::{
    application_element, snapshot_elements,
};

const WINDOW_WIDTH: f64 = 480.0;
const WINDOW_HEIGHT: f64 = 560.0;
const MARGIN: f64 = 20.0;
const LABEL_HEIGHT: f64 = 18.0;
const FIELD_HEIGHT: f64 = 28.0;
const ROW_GAP: f64 = 14.0;
const POLL_INTERVAL_MS: u64 = 100;

pub type LabFocusHandler = Arc<dyn Fn(&str) + Send + Sync>;

struct LabWindowState {
    window: Retained<NSWindow>,
    fields: Vec<(String, Retained<NSTextField>)>,
    last_focus_id: Option<String>,
}

thread_local! {
    static LAB_WINDOW: RefCell<Option<LabWindowState>> = const { RefCell::new(None) };
}

static FOCUS_HANDLER: Mutex<Option<LabFocusHandler>> = Mutex::new(None);
static POLL_EPOCH: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// Lab 切替テスト窓を前面に出す（未作成なら生成）。フォーカス変化は `on_focus` へ通知。
pub fn show(on_focus: LabFocusHandler) -> Result<(), String> {
    {
        let mut guard = FOCUS_HANDLER
            .lock()
            .map_err(|_| "lab focus handler lock poisoned".to_owned())?;
        *guard = Some(on_focus);
    }
    run_on_main_thread(show_on_main)
}

fn show_on_main() -> Result<(), String> {
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| "lab switch window requires the AppKit main thread".to_owned())?;

    LAB_WINDOW.with(|cell| {
        let mut guard = cell.borrow_mut();
        if let Some(state) = guard.as_ref() {
            activate_and_front(mtm, &state.window);
            if let Some((_, field)) = state.fields.first() {
                let _ = state
                    .window
                    .makeFirstResponder(Some(AsRef::<NSResponder>::as_ref(&**field)));
            }
            start_poll_loop();
            return Ok(());
        }

        let state = build_window(mtm);
        activate_and_front(mtm, &state.window);
        if let Some((_, field)) = state.fields.first() {
            let _ = state
                .window
                .makeFirstResponder(Some(AsRef::<NSResponder>::as_ref(&**field)));
        }
        *guard = Some(state);
        start_poll_loop();
        Ok(())
    })
}

fn activate_and_front(mtm: MainThreadMarker, window: &NSWindow) {
    let app = NSApplication::sharedApplication(mtm);
    #[allow(deprecated)]
    app.activateIgnoringOtherApps(true);
    window.makeKeyAndOrderFront(None::<&AnyObject>);
}

fn build_window(mtm: MainThreadMarker) -> LabWindowState {
    let content_rect = NSRect::new(
        NSPoint::new(0.0, 0.0),
        NSSize::new(WINDOW_WIDTH, WINDOW_HEIGHT),
    );
    let style = NSWindowStyleMask::Titled | NSWindowStyleMask::Closable;
    // SAFETY: MainThreadMarker 上で NSWindow を生成する。
    let window = unsafe {
        NSWindow::initWithContentRect_styleMask_backing_defer(
            NSWindow::alloc(mtm),
            content_rect,
            style,
            NSBackingStoreType::Buffered,
            false,
        )
    };
    unsafe {
        window.setReleasedWhenClosed(false);
    }
    window.setTitle(&NSString::from_str("ReadyType Lab Switch Test"));
    window.center();

    let content = NSView::initWithFrame(NSView::alloc(mtm), content_rect);
    let mut y = WINDOW_HEIGHT - MARGIN;
    let mut fields = Vec::with_capacity(LAB_SWITCH_FIELD_IDS.len());

    for (index, field_id) in LAB_SWITCH_FIELD_IDS.iter().enumerate() {
        let ordinal = index + 1;
        let language = if ordinal % 2 == 1 { "ja" } else { "en" };
        y -= LABEL_HEIGHT;
        let label = NSTextField::labelWithString(
            &NSString::from_str(&format!("Field {ordinal:02} ({language}) — {field_id}")),
            mtm,
        );
        AsRef::<NSView>::as_ref(&*label).setFrame(NSRect::new(
            NSPoint::new(MARGIN, y),
            NSSize::new(WINDOW_WIDTH - MARGIN * 2.0, LABEL_HEIGHT),
        ));
        content.addSubview(AsRef::<NSView>::as_ref(&*label));

        y -= 4.0 + FIELD_HEIGHT;
        let field = NSTextField::initWithFrame(
            NSTextField::alloc(mtm),
            NSRect::new(
                NSPoint::new(MARGIN, y),
                NSSize::new(WINDOW_WIDTH - MARGIN * 2.0, FIELD_HEIGHT),
            ),
        );
        field.setEditable(true);
        field.setBezeled(true);
        field.setBordered(true);
        AsRef::<NSControl>::as_ref(&*field).setStringValue(&NSString::from_str(""));
        field.setPlaceholderString(Some(&NSString::from_str(&format!(
            "focus → switch to {language}"
        ))));
        field.setAccessibilityIdentifier(Some(&NSString::from_str(field_id)));
        content.addSubview(AsRef::<NSView>::as_ref(&*field));
        fields.push(((*field_id).to_owned(), field));

        y -= ROW_GAP;
    }

    window.setContentView(Some(&content));
    LabWindowState {
        window,
        fields,
        last_focus_id: None,
    }
}

fn start_poll_loop() {
    let epoch = POLL_EPOCH.fetch_add(1, std::sync::atomic::Ordering::AcqRel) + 1;
    schedule_poll(epoch);
}

fn schedule_poll(epoch: u64) {
    dispatch_after_ms(POLL_INTERVAL_MS, move || {
        if POLL_EPOCH.load(std::sync::atomic::Ordering::Acquire) != epoch {
            return;
        }
        let cont = poll_once();
        if cont {
            schedule_poll(epoch);
        }
    });
}

fn poll_once() -> bool {
    LAB_WINDOW.with(|cell| {
        let mut guard = cell.borrow_mut();
        let Some(state) = guard.as_mut() else {
            return false;
        };
        if !state.window.isVisible() {
            return false;
        }
        let Some(id) = resolve_focused_lab_id(state) else {
            return true;
        };
        if state.last_focus_id.as_deref() == Some(id.as_str()) {
            return true;
        }
        state.last_focus_id = Some(id.clone());
        if let Ok(handler) = FOCUS_HANDLER.lock() {
            if let Some(handler) = handler.as_ref() {
                handler(&id);
            }
        }
        true
    })
}

fn resolve_focused_lab_id(state: &LabWindowState) -> Option<String> {
    if !state.window.isKeyWindow() {
        return None;
    }
    if let Some(id) = ax_lab_identifier() {
        return Some(id);
    }
    appkit_lab_identifier(state)
}

fn ax_lab_identifier() -> Option<String> {
    let pid = i32::try_from(std::process::id()).ok()?;
    let application = application_element(pid);
    let plan = AttributePlan::normal([AxAttribute::Role, AxAttribute::Identifier]);
    let elements = snapshot_elements(&application, &plan);
    for element in elements {
        let id = element.value(AxAttribute::Identifier);
        if LAB_SWITCH_FIELD_IDS.contains(&id) {
            return Some(id.to_owned());
        }
    }
    None
}

fn appkit_lab_identifier(state: &LabWindowState) -> Option<String> {
    let responder = state.window.firstResponder()?;
    let responder_ptr = Retained::as_ptr(&responder) as *const ();
    for (id, field) in &state.fields {
        if Retained::as_ptr(field) as *const () == responder_ptr {
            return Some(id.clone());
        }
        // field editor が firstResponder のとき
        // SAFETY: AppKit の fieldEditor:forObject:（作成せず取得のみ）。
        let editor = unsafe {
            state
                .window
                .fieldEditor_forObject(false, Some(field.as_ref()))
        };
        if let Some(editor) = editor {
            if Retained::as_ptr(&editor) as *const () == responder_ptr {
                return Some(id.clone());
            }
        }
    }
    None
}

fn dispatch_after_ms<F>(delay_ms: u64, work: F)
where
    F: FnOnce() + Send + 'static,
{
    struct Context<F> {
        work: Option<F>,
    }

    unsafe extern "C" fn invoke<F: FnOnce()>(context: *mut c_void) {
        // SAFETY: Box は schedule 側が exclusive に渡した。
        let mut context = unsafe { Box::from_raw(context.cast::<Context<F>>()) };
        if let Some(work) = context.work.take() {
            work();
        }
    }

    let delta_ns = i64::try_from(delay_ms.saturating_mul(1_000_000)).unwrap_or(i64::MAX);
    // SAFETY: DISPATCH_TIME_NOW + ns delta は dispatch_time の定石。
    let when = unsafe { dispatch_time(DISPATCH_TIME_NOW, delta_ns) };
    let context = Box::into_raw(Box::new(Context { work: Some(work) }));
    // SAFETY: main queue 上で invoke が一度だけ context を解放する。
    unsafe {
        dispatch_after_f(
            when,
            std::ptr::addr_of!(_dispatch_main_q),
            context.cast(),
            Some(invoke::<F>),
        );
    }
}

fn run_on_main_thread<F, R>(work: F) -> R
where
    F: FnOnce() -> R + Send,
    R: Send,
{
    if MainThreadMarker::new().is_some() {
        return work();
    }

    struct Context<F, R> {
        work: Option<F>,
        result: MaybeUninit<R>,
    }

    unsafe extern "C" fn invoke<F, R>(context: *mut c_void)
    where
        F: FnOnce() -> R,
    {
        // SAFETY: caller が exclusive に渡した Context。
        let context = unsafe { &mut *context.cast::<Context<F, R>>() };
        let work = context
            .work
            .take()
            .expect("lab window main-thread work missing");
        context.result.write(work());
    }

    let mut context = Context {
        work: Some(work),
        result: MaybeUninit::uninit(),
    };
    // SAFETY: main queue 上で invoke のみが context を触る。
    unsafe {
        dispatch_sync_f(
            std::ptr::addr_of!(_dispatch_main_q),
            std::ptr::addr_of_mut!(context).cast(),
            Some(invoke::<F, R>),
        );
    }
    // SAFETY: invoke が result を書いたあと。
    unsafe { context.result.assume_init() }
}

extern "C" {
    static mut _dispatch_main_q: c_void;

    fn dispatch_sync_f(
        queue: *const c_void,
        context: *mut c_void,
        work: Option<unsafe extern "C" fn(*mut c_void)>,
    );

    fn dispatch_time(when: u64, delta: i64) -> u64;
    fn dispatch_after_f(
        when: u64,
        queue: *const c_void,
        context: *mut c_void,
        work: Option<unsafe extern "C" fn(*mut c_void)>,
    );
}

const DISPATCH_TIME_NOW: u64 = 0;
