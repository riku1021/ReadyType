use std::{
    ffi::c_void,
    ptr::NonNull,
    sync::mpsc::{self, Receiver, Sender, TryRecvError},
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use block2::RcBlock;
use objc2::{rc::Retained, runtime::AnyObject};
use objc2_app_kit::{NSWorkspace, NSWorkspaceDidActivateApplicationNotification};
use objc2_application_services::{AXError, AXObserver, AXUIElement};
use objc2_core_foundation::{
    kCFRunLoopDefaultMode, CFRetained, CFRunLoop, CFRunLoopSource, CFString,
};

use crate::{
    application::ports::focus_monitor::{
        AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
    },
    domain::input_source::{AxAttribute, FocusSnapshot, Platform, PlatformAppId},
    infrastructure::platform::focus_monitor::{
        AttributePlan, MonitorState, MonitoringMode, ObserverRegistration, FALLBACK_INTERVAL_MS,
    },
};

use super::accessibility::{
    application_element, is_process_trusted, request_process_trust, snapshot_elements,
};

const FOCUSED_ELEMENT_CHANGED: &str = "AXFocusedUIElementChanged";
const PERMISSION_CHECK_INTERVAL: Duration = Duration::from_secs(1);
const CONSISTENCY_CHECK_INTERVAL: Duration = Duration::from_secs(2);

enum MonitorCommand {
    ActiveApplication(PlatformAppId),
    Evaluate { pid: i32, generation: u64 },
    SetDebug(bool),
    Stop,
}

struct CallbackContext {
    sender: Sender<MonitorCommand>,
    pid: i32,
    generation: u64,
}

struct ActiveObserver {
    _observer: CFRetained<AXObserver>,
    source: CFRetained<CFRunLoopSource>,
    _application: CFRetained<AXUIElement>,
    _context: Box<CallbackContext>,
}

impl ActiveObserver {
    fn unregister(self, run_loop: &CFRunLoop) {
        let mode = unsafe { kCFRunLoopDefaultMode };
        run_loop.remove_source(Some(&self.source), mode);
    }
}

pub struct MacOsFocusMonitor {
    sender: Option<Sender<MonitorCommand>>,
    worker: Option<JoinHandle<()>>,
    debug_capture: bool,
}

impl Default for MacOsFocusMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MacOsFocusMonitor {
    pub fn new() -> Self {
        Self {
            sender: None,
            worker: None,
            debug_capture: false,
        }
    }

    fn stop_worker(&mut self) -> Result<(), FocusMonitorError> {
        if let Some(sender) = self.sender.take() {
            let _ = sender.send(MonitorCommand::Stop);
        }
        if let Some(worker) = self.worker.take() {
            worker.join().map_err(|_| {
                FocusMonitorError::Operation("focus monitor worker panicked".to_owned())
            })?;
        }
        Ok(())
    }
}

impl Drop for MacOsFocusMonitor {
    fn drop(&mut self) {
        let _ = self.stop_worker();
    }
}

impl FocusMonitor for MacOsFocusMonitor {
    fn start(
        &mut self,
        attributes: Vec<AxAttribute>,
        sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError> {
        self.stop_worker()?;
        if !is_process_trusted() {
            return Err(FocusMonitorError::Operation(
                "accessibility permission is not granted".to_owned(),
            ));
        }
        let debug_capture = self.debug_capture;
        let (sender, receiver) = mpsc::channel();
        let worker_sender = sender.clone();
        let worker = thread::Builder::new()
            .name("readytype-focus-monitor".to_owned())
            .spawn(move || run_monitor(receiver, worker_sender, attributes, debug_capture, sink))
            .map_err(|error| FocusMonitorError::Operation(error.to_string()))?;
        self.sender = Some(sender);
        self.worker = Some(worker);
        Ok(())
    }

    fn stop(&mut self) -> Result<(), FocusMonitorError> {
        self.stop_worker()
    }

    fn set_debug_capture(&mut self, enabled: bool) -> Result<(), FocusMonitorError> {
        self.debug_capture = enabled;
        if let Some(sender) = &self.sender {
            sender
                .send(MonitorCommand::SetDebug(enabled))
                .map_err(|error| FocusMonitorError::Operation(error.to_string()))?;
        }
        Ok(())
    }

    fn permission_status(&self) -> AccessibilityPermission {
        if is_process_trusted() {
            AccessibilityPermission::Granted
        } else {
            AccessibilityPermission::Denied
        }
    }

    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError> {
        Ok(if request_process_trust() {
            AccessibilityPermission::Granted
        } else {
            AccessibilityPermission::Denied
        })
    }

    fn open_settings(&self) -> Result<(), FocusMonitorError> {
        std::process::Command::new("open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .spawn()
            .map_err(|error| FocusMonitorError::Operation(error.to_string()))?;
        Ok(())
    }
}

fn run_monitor(
    receiver: Receiver<MonitorCommand>,
    sender: Sender<MonitorCommand>,
    attributes: Vec<AxAttribute>,
    debug_capture: bool,
    sink: FocusEventSink,
) {
    let workspace = NSWorkspace::sharedWorkspace();
    let center = workspace.notificationCenter();
    let activation_sender = sender.clone();
    let activation_block = RcBlock::new(move |_| {
        if let Some(app) = frontmost_application() {
            let _ = activation_sender.send(MonitorCommand::ActiveApplication(app));
        }
    });
    // SAFETY: the block accepts the documented notification parameter and is retained by the center.
    let activation_observer: Retained<AnyObject> = unsafe {
        center
            .addObserverForName_object_queue_usingBlock(
                Some(NSWorkspaceDidActivateApplicationNotification),
                None,
                None,
                &activation_block,
            )
            .into()
    };

    let Some(run_loop) = CFRunLoop::current() else {
        return;
    };
    let mut state = MonitorState::default();
    let normal_plan = AttributePlan::normal(attributes);
    let mut debug = debug_capture;
    let mut observer = None;
    let mut last_fallback = Instant::now();
    let mut last_consistency = Instant::now();
    let mut last_permission_check = Instant::now();
    if let Some(app) = frontmost_application() {
        activate(
            app,
            &mut state,
            &mut observer,
            &run_loop,
            &sender,
            &normal_plan,
            debug,
            &sink,
        );
    }

    let mut running = true;
    while running {
        let mode = unsafe { kCFRunLoopDefaultMode };
        CFRunLoop::run_in_mode(mode, 0.01, true);
        loop {
            match receiver.try_recv() {
                Ok(MonitorCommand::ActiveApplication(app)) => activate(
                    app,
                    &mut state,
                    &mut observer,
                    &run_loop,
                    &sender,
                    &normal_plan,
                    debug,
                    &sink,
                ),
                Ok(MonitorCommand::Evaluate { pid, generation }) => {
                    if state.enqueue(pid, generation) {
                        evaluate_pending(&mut state, &normal_plan, debug, &sink);
                    }
                }
                Ok(MonitorCommand::SetDebug(enabled)) => debug = enabled,
                Ok(MonitorCommand::Stop) | Err(TryRecvError::Disconnected) => {
                    running = false;
                    break;
                }
                Err(TryRecvError::Empty) => break,
            }
        }

        if last_permission_check.elapsed() >= PERMISSION_CHECK_INTERVAL {
            let trusted = is_process_trusted();
            let was_stopped = state.mode() == MonitoringMode::StoppedNoPermission;
            state.update_permission(trusted);
            if !trusted {
                if let Some(active) = observer.take() {
                    active.unregister(&run_loop);
                }
            } else if was_stopped {
                if let Some(app) = frontmost_application() {
                    activate(
                        app,
                        &mut state,
                        &mut observer,
                        &run_loop,
                        &sender,
                        &normal_plan,
                        debug,
                        &sink,
                    );
                }
            }
            last_permission_check = Instant::now();
        }

        let fallback_due = matches!(state.mode(), MonitoringMode::FallbackPolling { .. })
            && last_fallback.elapsed() >= Duration::from_millis(FALLBACK_INTERVAL_MS);
        let consistency_due = state.mode() == MonitoringMode::AxNotification
            && last_consistency.elapsed() >= CONSISTENCY_CHECK_INTERVAL;
        if fallback_due || consistency_due {
            if let Some((pid, generation)) = state.active_identity() {
                if state.enqueue(pid, generation) {
                    evaluate_pending(&mut state, &normal_plan, debug, &sink);
                }
            }
            if fallback_due {
                last_fallback = Instant::now();
            }
            if consistency_due {
                last_consistency = Instant::now();
            }
        }
    }

    if let Some(active) = observer.take() {
        active.unregister(&run_loop);
    }
    // SAFETY: this is the exact observer token returned by this notification center.
    unsafe { center.removeObserver(&activation_observer) };
}

#[allow(clippy::too_many_arguments)]
fn activate(
    app: PlatformAppId,
    state: &mut MonitorState,
    observer: &mut Option<ActiveObserver>,
    run_loop: &CFRunLoop,
    sender: &Sender<MonitorCommand>,
    normal_plan: &AttributePlan,
    debug: bool,
    sink: &FocusEventSink,
) {
    if let Some(active) = observer.take() {
        active.unregister(run_loop);
    }
    let next_generation = state.generation().wrapping_add(1);
    match register_observer(app.pid(), next_generation, sender.clone(), run_loop) {
        Ok(active) => {
            state.activate(app, ObserverRegistration::Registered);
            *observer = Some(active);
        }
        Err(_) => state.activate(app, ObserverRegistration::Unsupported),
    }
    if let Some((pid, generation)) = state.active_identity() {
        if state.enqueue(pid, generation) {
            evaluate_pending(state, normal_plan, debug, sink);
        }
    }
}

fn register_observer(
    pid: i32,
    generation: u64,
    sender: Sender<MonitorCommand>,
    run_loop: &CFRunLoop,
) -> Result<ActiveObserver, AXError> {
    let application = application_element(pid);
    let mut observer_pointer = std::ptr::null_mut();
    // SAFETY: output pointer is valid and callback obeys the AX callback ABI.
    let error = unsafe {
        AXObserver::create(
            pid,
            Some(observer_callback),
            NonNull::from(&mut observer_pointer),
        )
    };
    if error != AXError::Success {
        return Err(error);
    }
    let observer_pointer = NonNull::new(observer_pointer).ok_or(AXError::Failure)?;
    // SAFETY: AXObserverCreate returned this create-rule reference successfully.
    let observer = unsafe { CFRetained::from_raw(observer_pointer) };
    let mut context = Box::new(CallbackContext {
        sender,
        pid,
        generation,
    });
    let notification = CFString::from_static_str(FOCUSED_ELEMENT_CHANGED);
    // SAFETY: context remains boxed and alive until the observer is removed and released.
    let error = unsafe {
        observer.add_notification(
            &application,
            &notification,
            (&mut *context as *mut CallbackContext).cast(),
        )
    };
    if error != AXError::Success && error != AXError::NotificationAlreadyRegistered {
        return Err(error);
    }
    // SAFETY: the observer owns a valid run-loop source for its lifetime.
    let source = unsafe { observer.run_loop_source() };
    let mode = unsafe { kCFRunLoopDefaultMode };
    run_loop.add_source(Some(&source), mode);
    Ok(ActiveObserver {
        _observer: observer,
        source,
        _application: application,
        _context: context,
    })
}

unsafe extern "C-unwind" fn observer_callback(
    _observer: NonNull<AXObserver>,
    _element: NonNull<AXUIElement>,
    _notification: NonNull<CFString>,
    refcon: *mut c_void,
) {
    let Some(context) = (unsafe { refcon.cast::<CallbackContext>().as_ref() }) else {
        return;
    };
    let _ = context.sender.send(MonitorCommand::Evaluate {
        pid: context.pid,
        generation: context.generation,
    });
}

fn evaluate_pending(
    state: &mut MonitorState,
    normal_plan: &AttributePlan,
    debug: bool,
    sink: &FocusEventSink,
) {
    let Some((pid, generation)) = state.take_pending() else {
        return;
    };
    let Some(app) = state.active_app().cloned() else {
        return;
    };
    if !state.accepts(pid, generation) {
        return;
    }
    let application = application_element(pid);
    let debug_plan;
    let plan = if debug {
        debug_plan = AttributePlan::debug();
        &debug_plan
    } else {
        normal_plan
    };
    sink(FocusSnapshot::new(
        app,
        generation,
        snapshot_elements(&application, plan),
    ));
}

fn frontmost_application() -> Option<PlatformAppId> {
    let application = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let bundle_id = application.bundleIdentifier()?.to_string();
    PlatformAppId::new(Platform::MacOs, &bundle_id, application.processIdentifier()).ok()
}
