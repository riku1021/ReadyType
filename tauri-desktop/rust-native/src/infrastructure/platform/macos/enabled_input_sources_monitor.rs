//! Observe enabled / installed input-source set changes and re-list after delays.
//!
//! Delivery channels (both needed on modern macOS):
//! - CF distributed: `kTISNotifyEnabledKeyboardInputSourcesChanged`
//! - NS distributed: `AppleEnabledInputSourcesChangedNotification`,
//!   `AppleInstalledInputSourcesChangedNotification`
//!
//! CF and NS distributed centers are **not** toll-free bridged. In-process
//! `TISCreateInputSourceList` can stay stale after Settings edits; the list
//! path uses a short-lived helper process for enabled IDs, and we re-query on
//! the main queue after ~100ms and ~500ms (Accessibility TCC pattern).
//!
//! Emitting to the WebView is wired at the composition root (callback).

use std::ffi::c_void;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use objc2::{define_class, msg_send, rc::Retained, sel, AnyThread, DefinedClass};
use objc2_foundation::{
    NSDistributedNotificationCenter, NSNotification, NSNotificationName,
    NSNotificationSuspensionBehavior, NSObject, NSObjectProtocol, NSString,
};

const RECHECK_DELAY_MS: u64 = 100;
const CONFIRM_DELAY_MS: u64 = 500;
const CF_NOTIFICATION_SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY: i32 = 4;
const DISPATCH_TIME_NOW: u64 = 0;

/// Unique observer identity for CFNotificationCenter (null observer is unreliable).
static CF_OBSERVER_TOKEN: u8 = 0;
/// NS+CF が同時発火しても再 list を畳む。
static RECHECK_EPOCH: AtomicU64 = AtomicU64::new(0);

type OnChange = Arc<dyn Fn() + Send + Sync + 'static>;

static SINK: Mutex<Option<OnChange>> = Mutex::new(None);

struct ObserverIvars {
    on_change: OnChange,
}

define_class!(
    // SAFETY:
    // - NSObject has no subclassing requirements.
    // - `EnabledInputSourcesNsObserver` does not implement `Drop`.
    #[unsafe(super(NSObject))]
    #[ivars = ObserverIvars]
    struct EnabledInputSourcesNsObserver;

    unsafe impl NSObjectProtocol for EnabledInputSourcesNsObserver {}

    impl EnabledInputSourcesNsObserver {
        #[unsafe(method(appleInputSourcesChanged:))]
        fn apple_input_sources_changed(&self, _notification: &NSNotification) {
            schedule_rechecks(Arc::clone(&self.ivars().on_change));
        }
    }
);

impl EnabledInputSourcesNsObserver {
    fn new(on_change: OnChange) -> Retained<Self> {
        let this = Self::alloc().set_ivars(ObserverIvars { on_change });
        // SAFETY: designated initializer of NSObject.
        unsafe { msg_send![super(this), init] }
    }
}

/// Invokes `on_change` on the main queue after ~100ms and again after ~500ms.
pub fn start_monitor(on_change: impl Fn() + Send + Sync + 'static) {
    let on_change: OnChange = Arc::new(on_change);
    {
        let Ok(mut guard) = SINK.lock() else {
            return;
        };
        *guard = Some(Arc::clone(&on_change));
    }

    register_cf_tis_observer();
    register_ns_apple_observers(on_change);
}

fn register_cf_tis_observer() {
    // SAFETY: CF distributed observer for Carbon TIS enabled-set changes.
    unsafe {
        let center = CFNotificationCenterGetDistributedCenter();
        if center.is_null() {
            eprintln!("input-source: CFNotificationCenterGetDistributedCenter returned null");
            return;
        }
        let name = kTISNotifyEnabledKeyboardInputSourcesChanged;
        if name.is_null() {
            eprintln!("input-source: kTISNotifyEnabledKeyboardInputSourcesChanged is null");
            return;
        }
        CFNotificationCenterAddObserver(
            center,
            std::ptr::from_ref(&CF_OBSERVER_TOKEN).cast_mut().cast(),
            Some(cf_notification_callback),
            name,
            std::ptr::null(),
            CF_NOTIFICATION_SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY,
        );
    }
}

fn register_ns_apple_observers(on_change: OnChange) {
    let observer = EnabledInputSourcesNsObserver::new(on_change);
    let center = NSDistributedNotificationCenter::defaultCenter();
    for name in [
        "AppleEnabledInputSourcesChangedNotification",
        "AppleInstalledInputSourcesChangedNotification",
    ] {
        let ns_name = NSString::from_str(name);
        let ns_name: &NSNotificationName = &ns_name;
        // SAFETY: observer implements the selector; DeliverImmediately for Accessory.
        unsafe {
            center.addObserver_selector_name_object_suspensionBehavior(
                observer.as_ref(),
                sel!(appleInputSourcesChanged:),
                Some(ns_name),
                None,
                NSNotificationSuspensionBehavior::DeliverImmediately,
            );
        }
    }
    // Keep the observer alive for process lifetime.
    std::mem::forget(observer);
}

unsafe extern "C" fn cf_notification_callback(
    _center: *mut c_void,
    _observer: *mut c_void,
    _name: *const c_void,
    _object: *const c_void,
    _user_info: *const c_void,
) {
    let on_change = match SINK.lock() {
        Ok(guard) => guard.as_ref().map(Arc::clone),
        Err(_) => None,
    };
    if let Some(on_change) = on_change {
        schedule_rechecks(on_change);
    }
}

fn schedule_rechecks(on_change: OnChange) {
    let epoch = RECHECK_EPOCH.fetch_add(1, Ordering::Relaxed) + 1;
    let first = Arc::clone(&on_change);
    dispatch_after_ms(RECHECK_DELAY_MS, move || {
        if RECHECK_EPOCH.load(Ordering::Relaxed) == epoch {
            first();
        }
    });
    dispatch_after_ms(CONFIRM_DELAY_MS, move || {
        if RECHECK_EPOCH.load(Ordering::Relaxed) == epoch {
            on_change();
        }
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

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISNotifyEnabledKeyboardInputSourcesChanged: *const c_void;
}

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFNotificationCenterGetDistributedCenter() -> *mut c_void;
    fn CFNotificationCenterAddObserver(
        center: *mut c_void,
        observer: *mut c_void,
        callback: Option<
            unsafe extern "C" fn(
                *mut c_void,
                *mut c_void,
                *const c_void,
                *const c_void,
                *const c_void,
            ),
        >,
        name: *const c_void,
        object: *const c_void,
        suspension_behavior: i32,
    );
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
