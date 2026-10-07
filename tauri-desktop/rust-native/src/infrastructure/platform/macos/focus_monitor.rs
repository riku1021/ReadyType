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
        SwitchingAttributeIndex,
    },
    domain::input_source::{AxAttribute, FocusSnapshot, Platform, PlatformAppId},
    infrastructure::platform::focus_monitor::{
        focus_fingerprint, is_expensive_ax_attribute, is_switch_candidate_role,
        role_may_need_description, AttributePlan, MonitorState, MonitoringMode,
        ObserverRegistration, FALLBACK_INTERVAL_MS, MAX_PARENT_DEPTH,
    },
};

use super::accessibility::{
    application_element, is_process_trusted, request_process_trust, snapshot_chain_with_depth,
    snapshot_elements,
};

const FOCUSED_ELEMENT_CHANGED: &str = "AXFocusedUIElementChanged";
const PERMISSION_CHECK_INTERVAL: Duration = Duration::from_secs(1);
const CONSISTENCY_CHECK_INTERVAL: Duration = Duration::from_secs(2);

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
        attribute_index: SwitchingAttributeIndex,
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
            .spawn(move || {
                run_monitor(
                    receiver,
                    worker_sender,
                    attribute_index,
                    debug_capture,
                    sink,
                )
            })
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
    attribute_index: SwitchingAttributeIndex,
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
            &attribute_index,
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
                    &attribute_index,
                    debug,
                    &sink,
                ),
                Ok(MonitorCommand::Evaluate { pid }) => {
                    if state.enqueue(pid) {
                        evaluate_pending(&mut state, &attribute_index, debug, &sink);
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
                        &attribute_index,
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
            if let Some(pid) = state.active_app().map(PlatformAppId::pid) {
                if state.enqueue(pid) {
                    evaluate_pending(&mut state, &attribute_index, debug, &sink);
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

fn plan_for_app(attribute_index: &SwitchingAttributeIndex, app_id: &str) -> AttributePlan {
    let attributes = attribute_index
        .iter()
        .find(|(id, _)| id == app_id)
        .map(|(_, attrs)| attrs.clone())
        .unwrap_or_default();
    AttributePlan::normal(attributes)
}

#[allow(clippy::too_many_arguments)]
fn activate(
    app: PlatformAppId,
    state: &mut MonitorState,
    observer: &mut Option<ActiveObserver>,
    run_loop: &CFRunLoop,
    sender: &Sender<MonitorCommand>,
    attribute_index: &SwitchingAttributeIndex,
    debug: bool,
    sink: &FocusEventSink,
) {
    if let Some(active) = observer.take() {
        active.unregister(run_loop);
    }
    match register_observer(app.pid(), sender.clone(), run_loop) {
        Ok(active) => {
            state.activate(app, ObserverRegistration::Registered);
            *observer = Some(active);
        }
        Err(_) => state.activate(app, ObserverRegistration::Unsupported),
    }
    if let Some(pid) = state.active_app().map(PlatformAppId::pid) {
        if state.enqueue(pid) {
            evaluate_pending(state, attribute_index, debug, sink);
        }
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
    attribute_index: &SwitchingAttributeIndex,
    debug: bool,
    sink: &FocusEventSink,
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
    let ax_started = Instant::now();
    let (elements, ax_micros, fingerprint) = if debug {
        let debug_plan = AttributePlan::debug();
        let elements = snapshot_elements(&application, &debug_plan);
        let ax_micros = u64::try_from(ax_started.elapsed().as_micros()).unwrap_or(u64::MAX);
        let fingerprint = focus_fingerprint(&elements);
        (elements, ax_micros, fingerprint)
    } else {
        let app_plan = plan_for_app(attribute_index, app.app_id());
        // 1) focused だけ取り、Role で候補判定（非候補は親ウォークしない）。
        let shallow =
            snapshot_chain_with_depth(&application, &AttributePlan::fingerprint(), 1, true);
        let focused_role = shallow
            .snapshots()
            .first()
            .map(|snapshot| snapshot.value(AxAttribute::Role).to_string())
            .unwrap_or_default();
        if is_switch_candidate_role(&focused_role) {
            // 2) 候補: shallow の focused を再利用して親だけ伸ばす（FOCUSED 再 IPC なし）。
            let mut chain =
                shallow.extend_parent_walk(&AttributePlan::fingerprint(), MAX_PARENT_DEPTH, true);
            let fingerprint = focus_fingerprint(chain.snapshots());
            if state.peek_focus_unchanged(&fingerprint) {
                return;
            }
            let mut enrich_attrs = app_plan.attributes_beyond_fingerprint();
            for attribute in [
                AxAttribute::Identifier,
                AxAttribute::DomIdentifier,
                AxAttribute::DomClassList,
            ] {
                if !enrich_attrs.contains(&attribute) {
                    enrich_attrs.push(attribute);
                }
            }
            if !role_may_need_description(&focused_role) {
                enrich_attrs.retain(|attribute| !is_expensive_ax_attribute(*attribute));
            }
            chain.enrich(&enrich_attrs);
            let elements = chain.into_snapshots();
            let ax_micros = u64::try_from(ax_started.elapsed().as_micros()).unwrap_or(u64::MAX);
            (elements, ax_micros, fingerprint)
        } else {
            // 非候補: focused のみ・enrich なし（Button/Link/Row 等）。
            let fingerprint = focus_fingerprint(shallow.snapshots());
            if state.peek_focus_unchanged(&fingerprint) {
                return;
            }
            let elements = shallow.into_snapshots();
            let ax_micros = u64::try_from(ax_started.elapsed().as_micros()).unwrap_or(u64::MAX);
            (elements, ax_micros, fingerprint)
        }
    };
    // 空フォーカスは一時失敗。generation を進めず sink もしない。
    if fingerprint == crate::domain::input_source::EMPTY_FOCUS_KEY {
        return;
    }
    let Some(generation) = state.commit_focus_change(&fingerprint) else {
        return;
    };
    sink(FocusSnapshot::new(app, generation, elements), ax_micros);
}

fn frontmost_application() -> Option<PlatformAppId> {
    let application = NSWorkspace::sharedWorkspace().frontmostApplication()?;
    let bundle_id = application.bundleIdentifier()?.to_string();
    PlatformAppId::new(Platform::MacOs, &bundle_id, application.processIdentifier()).ok()
}
