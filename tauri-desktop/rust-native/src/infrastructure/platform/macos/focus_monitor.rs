use std::{
    collections::HashSet,
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
        focus_fingerprint, AttributePlan, MonitorState, MonitoringMode, ObserverRegistration,
        FALLBACK_INTERVAL_MS,
    },
};

use super::accessibility::{
    application_element, is_process_trusted, request_chromium_accessibility, request_process_trust,
    snapshot_elements,
};

const FOCUSED_ELEMENT_CHANGED: &str = "AXFocusedUIElementChanged";
const PERMISSION_CHECK_INTERVAL: Duration = Duration::from_secs(1);
const CONSISTENCY_CHECK_INTERVAL: Duration = Duration::from_secs(2);
/// Chromium builds the AX tree asynchronously after AXManualAccessibility;
/// walk too early and AXFocusedUIElement stays empty.
const CHROMIUM_AX_SETTLE_MS: u64 = 500;

enum MonitorCommand {
    ActiveApplication(PlatformAppId),
    Evaluate { pid: i32 },
    SetDebug(bool),
    Stop,
}

struct CallbackContext {
    sender: Sender<MonitorCommand>,
    pid: i32,
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

    fn frontmost_app(&self) -> Option<PlatformAppId> {
        frontmost_application()
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
    let mut chromium_ax_settled: HashSet<i32> = HashSet::new();
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
            &mut chromium_ax_settled,
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
                    &mut chromium_ax_settled,
                ),
                Ok(MonitorCommand::Evaluate { pid }) => {
                    if state.enqueue(pid) {
                        evaluate_pending(
                            &mut state,
                            &normal_plan,
                            debug,
                            &sink,
                            &run_loop,
                            &mut chromium_ax_settled,
                        );
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
                        &mut chromium_ax_settled,
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
            if let Some(pid) = state.active_app().map(PlatformAppId::pid) {
                if state.enqueue(pid) {
                    evaluate_pending(
                        &mut state,
                        &normal_plan,
                        debug,
                        &sink,
                        &run_loop,
                        &mut chromium_ax_settled,
                    );
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
    chromium_ax_settled: &mut HashSet<i32>,
) {
    if let Some(active) = observer.take() {
        active.unregister(run_loop);
    }
    let pid = app.pid();
    ensure_chromium_accessibility(pid, run_loop, chromium_ax_settled);
    match register_observer(pid, sender.clone(), run_loop) {
        Ok(active) => {
            state.activate(app, ObserverRegistration::Registered);
            *observer = Some(active);
        }
        Err(_) => state.activate(app, ObserverRegistration::Unsupported),
    }
    if let Some(pid) = state.active_app().map(PlatformAppId::pid) {
        if state.enqueue(pid) {
            evaluate_pending(
                state,
                normal_plan,
                debug,
                sink,
                run_loop,
                chromium_ax_settled,
            );
        }
    }
}

/// Electron / Chromium: wake the web AX tree once per PID, then wait for async build.
fn ensure_chromium_accessibility(
    pid: i32,
    run_loop: &CFRunLoop,
    chromium_ax_settled: &mut HashSet<i32>,
) {
    let application = application_element(pid);
    if !request_chromium_accessibility(&application) {
        return;
    }
    if !chromium_ax_settled.insert(pid) {
        return;
    }
    let _ = run_loop; // settle on this thread's current run loop (same as monitor worker)
    pump_chromium_ax_settle();
}

fn pump_chromium_ax_settle() {
    let mode = unsafe { kCFRunLoopDefaultMode };
    let deadline = Instant::now() + Duration::from_millis(CHROMIUM_AX_SETTLE_MS);
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let slice = remaining.min(Duration::from_millis(50)).as_secs_f64();
        if slice <= 0.0 {
            break;
        }
        CFRunLoop::run_in_mode(mode, slice, true);
    }
}

fn register_observer(
    pid: i32,
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
    let mut context = Box::new(CallbackContext { sender, pid });
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
    let _ = context
        .sender
        .send(MonitorCommand::Evaluate { pid: context.pid });
}

fn evaluate_pending(
    state: &mut MonitorState,
    normal_plan: &AttributePlan,
    debug: bool,
    sink: &FocusEventSink,
    run_loop: &CFRunLoop,
    chromium_ax_settled: &mut HashSet<i32>,
) {
    let Some(pid) = state.take_pending() else {
        return;
    };
    let Some(app) = state.active_app().cloned() else {
        return;
    };
    if !state.accepts_pid(pid) {
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
    let mut elements = snapshot_elements(&application, plan);
    // 空のときは Chromium ツリー未起動の可能性がある。未 settle なら起こして一度だけ再試行。
    if elements.is_empty() && !chromium_ax_settled.contains(&pid) {
        ensure_chromium_accessibility(pid, run_loop, chromium_ax_settled);
        elements = snapshot_elements(&application, plan);
    }
    // 空フォーカスは sink しない（一時的な FOCUSED 失敗や WebView AX 欠落で
    // last_snapshot を潰し、Quick Set / 切替が死ぬのを防ぐ）。
    if elements.is_empty() {
        return;
    }
    let fingerprint = focus_fingerprint(&elements);
    let Some(generation) = state.commit_focus_change(&fingerprint) else {
        return;
    };
    sink(FocusSnapshot::new(app, generation, elements));
}

fn frontmost_application() -> Option<PlatformAppId> {
    let application = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let bundle_id = application.bundleIdentifier()?.to_string();
    PlatformAppId::new(Platform::MacOs, &bundle_id, application.processIdentifier()).ok()
}
