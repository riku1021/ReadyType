use std::{
    path::Path,
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use windows::{
    core::implement,
    Win32::{
        Foundation::{CloseHandle, HANDLE},
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
                COINIT_MULTITHREADED,
            },
            Threading::{
                OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT,
                PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
        UI::Accessibility::{
            CUIAutomation, IUIAutomation, IUIAutomationCacheRequest, IUIAutomationElement,
            IUIAutomationFocusChangedEventHandler, IUIAutomationFocusChangedEventHandler_Impl,
            UIA_AutomationIdPropertyId, UIA_ButtonControlTypeId, UIA_CheckBoxControlTypeId,
            UIA_ClassNamePropertyId, UIA_ComboBoxControlTypeId, UIA_ControlTypePropertyId,
            UIA_DocumentControlTypeId, UIA_EditControlTypeId, UIA_HelpTextPropertyId,
            UIA_ListControlTypeId, UIA_LocalizedControlTypePropertyId, UIA_NamePropertyId,
            UIA_PaneControlTypeId, UIA_ProcessIdPropertyId, UIA_TextControlTypeId,
            UIA_TreeControlTypeId, UIA_WindowControlTypeId,
        },
        UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId},
    },
};

use crate::{
    application::ports::focus_monitor::{
        AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
    },
    domain::input_source::{
        AxAttribute, AxElementSnapshot, FocusSnapshot, Platform, PlatformAppId,
    },
    infrastructure::platform::focus_monitor::{focus_fingerprint, MAX_PARENT_DEPTH},
};

const CONSISTENCY_INTERVAL: Duration = Duration::from_secs(2);
const FALLBACK_INTERVAL: Duration = Duration::from_millis(50);

enum MonitorCommand {
    Evaluate,
    SetDebug(bool),
    Stop,
}

#[implement(IUIAutomationFocusChangedEventHandler)]
struct FocusChangedHandler {
    sender: Sender<MonitorCommand>,
}

impl IUIAutomationFocusChangedEventHandler_Impl for FocusChangedHandler_Impl {
    fn HandleFocusChangedEvent(
        &self,
        _sender: windows::core::Ref<IUIAutomationElement>,
    ) -> windows::core::Result<()> {
        let _ = self.sender.send(MonitorCommand::Evaluate);
        Ok(())
    }
}

pub struct WindowsFocusMonitor {
    sender: Option<Sender<MonitorCommand>>,
    worker: Option<JoinHandle<()>>,
    debug_capture: bool,
}

impl Default for WindowsFocusMonitor {
    fn default() -> Self {
        Self {
            sender: None,
            worker: None,
            debug_capture: false,
        }
    }
}

impl WindowsFocusMonitor {
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

impl Drop for WindowsFocusMonitor {
    fn drop(&mut self) {
        let _ = self.stop_worker();
    }
}

impl FocusMonitor for WindowsFocusMonitor {
    fn start(
        &mut self,
        attributes: Vec<AxAttribute>,
        sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError> {
        self.stop_worker()?;
        let debug_capture = self.debug_capture;
        let (sender, receiver) = mpsc::channel();
        let (startup_sender, startup_receiver) = mpsc::sync_channel(1);
        let worker_sender = sender.clone();
        let worker = thread::Builder::new()
            .name("readytype-windows-focus-monitor".to_owned())
            .spawn(move || {
                run_monitor(
                    receiver,
                    worker_sender,
                    startup_sender,
                    attributes,
                    debug_capture,
                    sink,
                )
            })
            .map_err(|error| FocusMonitorError::Operation(error.to_string()))?;
        match startup_receiver.recv() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => {
                let _ = sender.send(MonitorCommand::Stop);
                let _ = worker.join();
                return Err(FocusMonitorError::Operation(error));
            }
            Err(error) => {
                let _ = worker.join();
                return Err(FocusMonitorError::Operation(error.to_string()));
            }
        }
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
        AccessibilityPermission::Granted
    }

    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError> {
        Ok(AccessibilityPermission::Granted)
    }

    fn open_settings(&self) -> Result<(), FocusMonitorError> {
        Ok(())
    }

    fn frontmost_app(&self) -> Option<PlatformAppId> {
        foreground_app()
    }
}

fn run_monitor(
    receiver: Receiver<MonitorCommand>,
    sender: Sender<MonitorCommand>,
    startup: mpsc::SyncSender<Result<(), String>>,
    attributes: Vec<AxAttribute>,
    mut debug_capture: bool,
    sink: FocusEventSink,
) {
    let _com = match ComScope::new() {
        Ok(com) => com,
        Err(error) => {
            let _ = startup.send(Err(format!("COM initialization failed: {error}")));
            return;
        }
    };
    let automation = match automation() {
        Ok(automation) => automation,
        Err(error) => {
            let _ = startup.send(Err(format!("UI Automation initialization failed: {error}")));
            return;
        }
    };
    let cache_request = match focus_cache_request(&automation) {
        Ok(cache_request) => cache_request,
        Err(error) => {
            let _ = startup.send(Err(format!(
                "UI Automation cache initialization failed: {error}"
            )));
            return;
        }
    };
    let handler: IUIAutomationFocusChangedEventHandler = FocusChangedHandler { sender }.into();
    // SAFETY: the handler remains alive until it is removed below; no cache request is needed.
    let event_registered = unsafe {
        automation.AddFocusChangedEventHandler(None::<&IUIAutomationCacheRequest>, &handler)
    }
    .is_ok();
    let _ = startup.send(Ok(()));
    let mut generation = 0;
    let mut last_fingerprint = String::new();
    evaluate_focus(
        &automation,
        &cache_request,
        &attributes,
        debug_capture,
        &sink,
        &mut generation,
        &mut last_fingerprint,
    );

    loop {
        let timeout = if event_registered {
            CONSISTENCY_INTERVAL
        } else {
            FALLBACK_INTERVAL
        };
        match receiver.recv_timeout(timeout) {
            Ok(MonitorCommand::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Ok(MonitorCommand::Evaluate) | Err(RecvTimeoutError::Timeout) => {
                evaluate_focus(
                    &automation,
                    &cache_request,
                    &attributes,
                    debug_capture,
                    &sink,
                    &mut generation,
                    &mut last_fingerprint,
                );
            }
            Ok(MonitorCommand::SetDebug(enabled)) => {
                debug_capture = enabled;
                last_fingerprint.clear();
                evaluate_focus(
                    &automation,
                    &cache_request,
                    &attributes,
                    debug_capture,
                    &sink,
                    &mut generation,
                    &mut last_fingerprint,
                );
            }
        }
    }

    if event_registered {
        // SAFETY: this is the same handler registered above and is still alive.
        let _ = unsafe { automation.RemoveFocusChangedEventHandler(&handler) };
    }
}

fn evaluate_focus(
    automation: &IUIAutomation,
    cache_request: &IUIAutomationCacheRequest,
    attributes: &[AxAttribute],
    debug_capture: bool,
    sink: &FocusEventSink,
    generation: &mut u64,
    last_fingerprint: &mut String,
) {
    let Some((app, elements)) =
        focused_snapshot(automation, cache_request, attributes, debug_capture)
    else {
        return;
    };
    let fingerprint = format!(
        "{}:{}:{}:{}:{}",
        app.app_id(),
        app.pid(),
        focus_fingerprint(&elements),
        elements
            .first()
            .map(|element| element.value(AxAttribute::Title))
            .unwrap_or_default(),
        elements
            .first()
            .map(|element| element.value(AxAttribute::Placeholder))
            .unwrap_or_default()
    );
    if *last_fingerprint == fingerprint {
        return;
    }
    *last_fingerprint = fingerprint;
    *generation = generation.wrapping_add(1);
    sink(FocusSnapshot::new(app, *generation, elements));
}

fn focused_snapshot(
    automation: &IUIAutomation,
    cache_request: &IUIAutomationCacheRequest,
    attributes: &[AxAttribute],
    debug_capture: bool,
) -> Option<(PlatformAppId, Vec<AxElementSnapshot>)> {
    // SAFETY: UI Automation returns the focused element and cached ancestor properties.
    let focused = unsafe { automation.GetFocusedElementBuildCache(cache_request) }.ok()?;
    // SAFETY: ProcessId was included in the cache request.
    let process_id = unsafe { focused.CachedProcessId() }.ok()?;
    let app_id = process_image_name(process_id as u32)?;
    let app = PlatformAppId::new(Platform::Windows, &app_id, process_id).ok()?;
    let mut element = Some(focused);
    let mut elements = Vec::new();
    for _ in 0..MAX_PARENT_DEPTH {
        let Some(current) = element.take() else {
            break;
        };
        let mut snapshot = AxElementSnapshot::new();
        for attribute in attributes.iter().copied() {
            if let Some(value) = cached_attribute_value(&current, attribute) {
                snapshot = snapshot.with(attribute, value);
            }
        }
        if debug_capture {
            for attribute in [
                AxAttribute::Role,
                AxAttribute::Subrole,
                AxAttribute::RoleDescription,
                AxAttribute::Title,
                AxAttribute::Description,
                AxAttribute::Identifier,
                AxAttribute::DomIdentifier,
                AxAttribute::DomClassList,
                AxAttribute::Help,
                AxAttribute::Placeholder,
            ] {
                if attributes.contains(&attribute) {
                    continue;
                }
                if let Some(value) = cached_attribute_value(&current, attribute) {
                    snapshot = snapshot.with(attribute, value);
                }
            }
        }
        elements.push(snapshot);
        // SAFETY: the cache request included all ancestors in the raw view.
        element = unsafe { current.GetCachedParent() }.ok();
    }
    Some((app, elements))
}

fn cached_attribute_value(
    element: &IUIAutomationElement,
    attribute: AxAttribute,
) -> Option<String> {
    // SAFETY: each corresponding property was requested before building the UIA cache.
    unsafe {
        match attribute {
            AxAttribute::Role => element
                .CachedControlType()
                .ok()
                .map(|control_type| control_type_role(control_type.0).to_owned()),
            AxAttribute::Subrole | AxAttribute::RoleDescription => element
                .CachedLocalizedControlType()
                .ok()
                .map(|value| value.to_string()),
            AxAttribute::Title => element.CachedName().ok().map(|value| value.to_string()),
            AxAttribute::Description | AxAttribute::Help | AxAttribute::Placeholder => {
                element.CachedHelpText().ok().map(|value| value.to_string())
            }
            AxAttribute::Identifier | AxAttribute::DomIdentifier => element
                .CachedAutomationId()
                .ok()
                .map(|value| value.to_string()),
            AxAttribute::DomClassList => element
                .CachedClassName()
                .ok()
                .map(|value| value.to_string()),
        }
    }
}

fn focus_cache_request(
    automation: &IUIAutomation,
) -> Result<IUIAutomationCacheRequest, windows::core::Error> {
    // SAFETY: the cache request is configured on the UIA worker thread before use.
    let cache_request = unsafe { automation.CreateCacheRequest()? };
    for property in [
        UIA_ProcessIdPropertyId,
        UIA_ControlTypePropertyId,
        UIA_LocalizedControlTypePropertyId,
        UIA_NamePropertyId,
        UIA_HelpTextPropertyId,
        UIA_AutomationIdPropertyId,
        UIA_ClassNamePropertyId,
    ] {
        // SAFETY: these are standard UIA property identifiers supported by cache requests.
        unsafe { cache_request.AddProperty(property)? };
    }
    // SAFETY: use the same raw tree as the existing parent traversal.
    let raw_view = unsafe { automation.RawViewCondition()? };
    unsafe {
        cache_request.SetTreeFilter(&raw_view)?;
        cache_request.SetTreeScope(windows::Win32::UI::Accessibility::TreeScope(
            windows::Win32::UI::Accessibility::TreeScope_Element.0
                | windows::Win32::UI::Accessibility::TreeScope_Ancestors.0,
        ))?;
    }
    Ok(cache_request)
}

fn control_type_role(control_type: i32) -> &'static str {
    match control_type {
        value if value == UIA_EditControlTypeId.0 => "AXTextField",
        value if value == UIA_DocumentControlTypeId.0 => "AXTextArea",
        value if value == UIA_TextControlTypeId.0 => "AXStaticText",
        value if value == UIA_ButtonControlTypeId.0 => "AXButton",
        value if value == UIA_CheckBoxControlTypeId.0 => "AXCheckBox",
        value if value == UIA_ComboBoxControlTypeId.0 => "AXComboBox",
        value if value == UIA_ListControlTypeId.0 => "AXList",
        value if value == UIA_TreeControlTypeId.0 => "AXOutline",
        value if value == UIA_WindowControlTypeId.0 => "AXWindow",
        value if value == UIA_PaneControlTypeId.0 => "AXGroup",
        _ => "AXUnknown",
    }
}

fn automation() -> Result<IUIAutomation, windows::core::Error> {
    // SAFETY: CUIAutomation is the documented UI Automation client COM class.
    unsafe { CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER) }
}

pub(crate) fn foreground_app() -> Option<PlatformAppId> {
    // SAFETY: the foreground window handle is queried from the current desktop session.
    let window = unsafe { GetForegroundWindow() };
    let mut process_id = 0;
    // SAFETY: process_id is a valid writable output pointer.
    let thread_id = unsafe { GetWindowThreadProcessId(window, Some(&mut process_id)) };
    if thread_id == 0 || process_id == 0 {
        return None;
    }
    PlatformAppId::new(
        Platform::Windows,
        &process_image_name(process_id)?,
        process_id as i32,
    )
    .ok()
}

fn process_image_name(process_id: u32) -> Option<String> {
    // SAFETY: OpenProcess is read-only and uses the minimum query permission.
    let process =
        unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, process_id) }.ok()?;
    let image = query_process_image_name(process);
    // SAFETY: this handle was returned by OpenProcess and is no longer needed.
    let _ = unsafe { CloseHandle(HANDLE(process.0)) };
    let name = Path::new(&image?)
        .file_name()?
        .to_str()?
        .to_ascii_lowercase();
    Some(name)
}

fn query_process_image_name(process: windows::Win32::Foundation::HANDLE) -> Option<String> {
    let mut buffer = [0u16; 32_768];
    let mut length = buffer.len() as u32;
    // SAFETY: `buffer` is writable and length is initialized to its capacity.
    unsafe {
        QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_FORMAT(0),
            windows::core::PWSTR(buffer.as_mut_ptr()),
            &mut length,
        )
    }
    .ok()?;
    Some(String::from_utf16_lossy(&buffer[..length as usize]))
}

struct ComScope {
    initialized: bool,
}

impl ComScope {
    fn new() -> Result<Self, windows::core::Error> {
        // SAFETY: this thread uses a single MTA COM apartment for UIA calls.
        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_ok() {
            Ok(Self { initialized: true })
        } else if result.0 as u32 == 0x8001_0106 {
            Ok(Self { initialized: false })
        } else {
            Err(windows::core::Error::from_hresult(result))
        }
    }
}

impl Drop for ComScope {
    fn drop(&mut self) {
        if self.initialized {
            // SAFETY: this scope exists only after successful COM initialization.
            unsafe { CoUninitialize() };
        }
    }
}
