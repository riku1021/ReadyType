use std::{ffi::c_void, ptr::NonNull};

use objc2_core_foundation::{
    CFDictionary, CFNotificationCenter, CFNotificationName, CFNotificationSuspensionBehavior,
    CFString,
};

use crate::application::ports::current_input_source_change::{
    CurrentInputSourceChangeMonitor, CurrentInputSourceChangeSink,
};

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    static kTISNotifySelectedKeyboardInputSourceChanged: *const CFString;
}

struct SinkContext {
    on_change: CurrentInputSourceChangeSink,
}

/// `kTISNotifySelectedKeyboardInputSourceChanged` で current キャッシュを invalidate。
pub struct MacOsCurrentInputSourceChangeMonitor {
    context: Option<NonNull<SinkContext>>,
}

// SAFETY: SinkContext は Send+Sync な Arc コールバックのみ保持。所有は monitor が独占し、
// CF コールバックと stop 以外からは触らない。
unsafe impl Send for MacOsCurrentInputSourceChangeMonitor {}

impl Default for MacOsCurrentInputSourceChangeMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOsCurrentInputSourceChangeMonitor {
    pub fn new() -> Self {
        Self { context: None }
    }
}

impl CurrentInputSourceChangeMonitor for MacOsCurrentInputSourceChangeMonitor {
    fn start(&mut self, sink: CurrentInputSourceChangeSink) -> Result<(), String> {
        self.stop();
        let Some(center) = CFNotificationCenter::distributed_center() else {
            return Err("CFNotificationCenter distributed center unavailable".to_owned());
        };
        let context = Box::into_raw(Box::new(SinkContext { on_change: sink }));
        let context = NonNull::new(context).expect("Box::into_raw is non-null");
        // SAFETY: Carbon が公開する CFString 定数。プロセス生存中有効。
        let name = unsafe { &*kTISNotifySelectedKeyboardInputSourceChanged };
        // SAFETY: observer は SinkContext。callback は observer を SinkContext として読む。
        // object=null で全送信者。DeliverImmediately で invalidate を遅らせない。
        unsafe {
            center.add_observer(
                context.as_ptr().cast(),
                Some(notification_callback),
                Some(name),
                std::ptr::null(),
                CFNotificationSuspensionBehavior::DeliverImmediately,
            );
        }
        self.context = Some(context);
        Ok(())
    }

    fn stop(&mut self) {
        let Some(context) = self.context.take() else {
            return;
        };
        if let Some(center) = CFNotificationCenter::distributed_center() {
            // SAFETY: start で登録した同一 observer ポインタ。
            unsafe {
                center.remove_every_observer(context.as_ptr().cast());
            }
        }
        // SAFETY: start で Box::into_raw したポインタを一度だけ戻す。
        drop(unsafe { Box::from_raw(context.as_ptr()) });
    }
}

impl Drop for MacOsCurrentInputSourceChangeMonitor {
    fn drop(&mut self) {
        self.stop();
    }
}

unsafe extern "C-unwind" fn notification_callback(
    _center: *mut CFNotificationCenter,
    observer: *mut c_void,
    _name: *const CFNotificationName,
    _object: *const c_void,
    _user_info: *const CFDictionary,
) {
    let Some(observer) = NonNull::new(observer) else {
        return;
    };
    // SAFETY: add_observer に渡した SinkContext。
    let context = unsafe { observer.cast::<SinkContext>().as_ref() };
    (context.on_change)();
}
