//! Observe `com.apple.accessibility.api` and re-check trust after short delays.
//!
//! macOS has no public Accessibility-permission change API. HIServices posts this
//! undocumented distributed notification when the Accessibility list changes
//! (any app). Immediate `AXIsProcessTrusted` reads are often stale; community
//! practice is to re-query on the main queue after a short delay, optionally
//! confirming again (~500ms) for fast toggles.
//!
//! Registration uses `NSNotificationSuspensionBehaviorDeliverImmediately` so
//! Accessory / background apps still receive the notification while delivery
//! would otherwise be suspended.
//!
//! Emitting to the WebView is wired at the composition root (callback), so this
//! module stays free of Tauri types.

use std::{ffi::c_void, sync::Arc};

use objc2::{define_class, msg_send, rc::Retained, sel, AnyThread, DefinedClass};
use objc2_foundation::{
    NSDistributedNotificationCenter, NSNotification, NSNotificationName,
    NSNotificationSuspensionBehavior, NSObject, NSObjectProtocol, NSString,
};

const ACCESSIBILITY_API_NOTIFICATION: &str = "com.apple.accessibility.api";
/// First re-read after the notification (common community delay).
const RECHECK_DELAY_MS: u64 = 100;
/// Second confirm for fast Settings toggles (Stack Overflow recommendation).
const CONFIRM_DELAY_MS: u64 = 500;

type OnChange = Arc<dyn Fn() + Send + Sync + 'static>;

struct ObserverIvars {
    on_change: OnChange,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - `AccessibilityPermissionObserver` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[ivars = ObserverIvars]
    struct AccessibilityPermissionObserver;

    unsafe impl NSObjectProtocol for AccessibilityPermissionObserver {}

    impl AccessibilityPermissionObserver {
        #[unsafe(method(accessibilityApiChanged:))]
        fn accessibility_api_changed(&self, _notification: &NSNotification) {
            schedule_rechecks(Arc::clone(&self.ivars().on_change));
        }
    }
);

impl AccessibilityPermissionObserver {
    fn new(on_change: OnChange) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ObserverIvars { on_change });
        // SAFETY: designated initializer of NSObject.
        unsafe { msg_send![super(this), init] }
    }
}

/// Invokes `on_change` on the main queue after ~100ms and again after ~500ms.
pub fn start_monitor(on_change: impl Fn() + Send + Sync + 'static) {
    let on_change: OnChange = Arc::new(on_change);
    let observer = AccessibilityPermissionObserver::new(on_change);
    let center = NSDistributedNotificationCenter::defaultCenter();
    let name = NSString::from_str(ACCESSIBILITY_API_NOTIFICATION);
    let name: &NSNotificationName = &name;
    // SAFETY: observer implements the selector; DeliverImmediately matches AX watch practice.
    unsafe {
        center.addObserver_selector_name_object_suspensionBehavior(
            observer.as_ref(),
            sel!(accessibilityApiChanged:),
            Some(name),
            None,
            NSNotificationSuspensionBehavior::DeliverImmediately,
        );
    }
    // Keep the observer alive for process lifetime (center does not retain strongly
    // for selector-based registration the way the block API does).
    std::mem::forget(observer);
}

fn schedule_rechecks(on_change: OnChange) {
    let first = Arc::clone(&on_change);
    dispatch_after_ms(RECHECK_DELAY_MS, move || {
        first();
    });
    dispatch_after_ms(CONFIRM_DELAY_MS, move || {
        on_change();
    });
}

fn dispatch_after_ms<F>(delay_ms: u64, work: F)
where
    F: FnOnce() + Send + 'static,
{
    struct Context<F> {
        work: Option<F>,
    }

    unsafe extern "C" fn invoke<F: FnOnce()>(context: *mut c_void) {
        // SAFETY: Box is exclusive ownership from schedule side.
        let mut context = unsafe { Box::from_raw(context.cast::<Context<F>>()) };
        if let Some(work) = context.work.take() {
            work();
        }
    }

    let delta_ns = i64::try_from(delay_ms.saturating_mul(1_000_000)).unwrap_or(i64::MAX);
    // SAFETY: DISPATCH_TIME_NOW + ns delta is the documented dispatch_time pattern.
    let when = unsafe { dispatch_time(DISPATCH_TIME_NOW, delta_ns) };
    let context = Box::into_raw(Box::new(Context { work: Some(work) }));
    // SAFETY: main queue invokes once and frees context.
    unsafe {
        dispatch_after_f(
            when,
            std::ptr::addr_of!(_dispatch_main_q),
            context.cast(),
            Some(invoke::<F>),
        );
    }
}

extern "C" {
    static mut _dispatch_main_q: c_void;

    fn dispatch_time(when: u64, delta: i64) -> u64;
    fn dispatch_after_f(
        when: u64,
        queue: *const c_void,
        context: *mut c_void,
        work: Option<unsafe extern "C" fn(*mut c_void)>,
    );
}

const DISPATCH_TIME_NOW: u64 = 0;
