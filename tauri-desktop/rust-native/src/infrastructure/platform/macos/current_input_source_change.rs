//! Subscribe to macOS selected-input-source changes (PROTO_TIS_NOTIFY).

use std::ffi::c_void;
use std::sync::Mutex;

use crate::application::ports::current_input_source_change::{
    CurrentInputSourceChangeMonitor, CurrentInputSourceChangeSink,
};

static SINK: Mutex<Option<CurrentInputSourceChangeSink>> = Mutex::new(None);

#[derive(Default)]
pub struct MacOsCurrentInputSourceChangeMonitor {
    running: bool,
}

impl CurrentInputSourceChangeMonitor for MacOsCurrentInputSourceChangeMonitor {
    fn start(&mut self, sink: CurrentInputSourceChangeSink) -> Result<(), String> {
        {
            let mut guard = SINK
                .lock()
                .map_err(|_| "tis notify sink lock poisoned".to_owned())?;
            *guard = Some(sink);
        }
        if self.running {
            return Ok(());
        }
        // SAFETY: CFNotificationCenter distributed observer for TIS selection changes.
        unsafe {
            let center = CFNotificationCenterGetDistributedCenter();
            if center.is_null() {
                return Err("CFNotificationCenterGetDistributedCenter returned null".to_owned());
            }
            let name = cfstr("com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged");
            CFNotificationCenterAddObserver(
                center,
                std::ptr::null_mut(),
                Some(notification_callback),
                name,
                std::ptr::null(),
                CF_NOTIFICATION_SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY,
            );
        }
        self.running = true;
        Ok(())
    }

    fn stop(&mut self) -> Result<(), String> {
        if !self.running {
            return Ok(());
        }
        unsafe {
            let center = CFNotificationCenterGetDistributedCenter();
            if !center.is_null() {
                let name = cfstr("com.apple.Carbon.TISNotifySelectedKeyboardInputSourceChanged");
                CFNotificationCenterRemoveObserver(
                    center,
                    std::ptr::null_mut(),
                    name,
                    std::ptr::null(),
                );
            }
        }
        if let Ok(mut guard) = SINK.lock() {
            *guard = None;
        }
        self.running = false;
        Ok(())
    }
}

unsafe extern "C" fn notification_callback(
    _center: *mut c_void,
    _observer: *mut c_void,
    _name: *const c_void,
    _object: *const c_void,
    _user_info: *const c_void,
) {
    if let Ok(guard) = SINK.lock() {
        if let Some(sink) = guard.as_ref() {
            sink();
        }
    }
}

unsafe fn cfstr(value: &str) -> *const c_void {
    CFStringCreateWithBytes(
        std::ptr::null(),
        value.as_ptr(),
        value.len() as isize,
        CF_STRING_ENCODING_UTF8,
        0,
    )
}

const CF_NOTIFICATION_SUSPENSION_BEHAVIOR_DELIVER_IMMEDIATELY: i32 = 4;
const CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
extern "C" {
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
    fn CFNotificationCenterRemoveObserver(
        center: *mut c_void,
        observer: *mut c_void,
        name: *const c_void,
        object: *const c_void,
    );
    fn CFStringCreateWithBytes(
        alloc: *const c_void,
        bytes: *const u8,
        num_bytes: isize,
        encoding: u32,
        is_external: u8,
    ) -> *const c_void;
}
