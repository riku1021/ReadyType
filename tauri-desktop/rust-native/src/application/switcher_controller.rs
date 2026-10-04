use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::domain::input_source::{
    default_rules, AxAttribute, FocusRule, FocusSnapshot, InputSourceId, LanguageTag, PlatformAppId,
};

use super::{
    input_source_switcher::{
        switch_input_source, ApplicationError, InputSourceSwitcher, PreparedFocus,
    },
    ports::{
        app_display_name::AppDisplayNameResolver,
        clock::Clock,
        focus_monitor::{AccessibilityPermission, FocusMonitor, FocusMonitorError},
        input_source::{AvailableInputSource, InputSourceGateway},
        settings_repository::SettingsRepository,
        switcher_events::SwitcherEventPublisher,
    },
    quick_set::{
        build_created_rule, build_quick_set_session, build_updated_rule, next_quick_set_priority,
        resolve_quick_set_name, QuickSetSession,
    },
    settings::SwitcherSettings,
    switcher_state::{SwitchLogEntry, SwitcherState},
    ui_locale_cache::UiLocaleCache,
};

type RuntimeSwitcher = InputSourceSwitcher<
    Box<dyn InputSourceGateway>,
    Arc<dyn SettingsRepository>,
    Arc<dyn Clock>,
    Arc<dyn SwitcherEventPublisher>,
>;

#[derive(Debug)]
pub enum ControllerError {
    Application(ApplicationError),
    Monitor(FocusMonitorError),
    InvalidRequest(String),
    WorkerStopped,
}

impl fmt::Display for ControllerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Application(error) => error.fmt(formatter),
            Self::Monitor(error) => write!(formatter, "focus monitor error: {error:?}"),
            Self::InvalidRequest(message) => write!(formatter, "invalid request: {message}"),
            Self::WorkerStopped => write!(formatter, "focus worker has stopped"),
        }
    }
}

impl std::error::Error for ControllerError {}

impl From<ApplicationError> for ControllerError {
    fn from(value: ApplicationError) -> Self {
        Self::Application(value)
    }
}

impl From<FocusMonitorError> for ControllerError {
    fn from(value: FocusMonitorError) -> Self {
        Self::Monitor(value)
    }
}

#[derive(Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix_ms(&self) -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis()
            .try_into()
            .unwrap_or(u64::MAX)
    }
}

enum WorkerMessage {
    Focus(FocusSnapshot),
    SettingsChanged,
    Stop,
}

pub struct SwitcherController {
    switcher: Arc<Mutex<RuntimeSwitcher>>,
    monitor: Mutex<Box<dyn FocusMonitor>>,
    sender: Sender<WorkerMessage>,
    worker: Mutex<Option<JoinHandle<()>>>,
    monitoring: AtomicBool,
    debug_capture: AtomicBool,
    last_snapshot: Arc<Mutex<Option<FocusSnapshot>>>,
    quick_set_session: Mutex<Option<QuickSetSession>>,
    publisher: Arc<dyn SwitcherEventPublisher>,
    ui_locale: Arc<UiLocaleCache>,
    app_display_names: Arc<dyn AppDisplayNameResolver>,
}

impl SwitcherController {
    pub fn new(
        settings: SwitcherSettings,
        gateway: Box<dyn InputSourceGateway>,
        repository: Arc<dyn SettingsRepository>,
        clock: Arc<dyn Clock>,
        publisher: Arc<dyn SwitcherEventPublisher>,
        monitor: Box<dyn FocusMonitor>,
        app_display_names: Arc<dyn AppDisplayNameResolver>,
    ) -> Result<Arc<Self>, ControllerError> {
        let resolved = SwitcherSettings::resolve_ui_locale(&settings.ui_locale);
        let ui_locale = Arc::new(UiLocaleCache::new(&resolved));
        let switcher = Arc::new(Mutex::new(InputSourceSwitcher::new(
            settings,
            repository_gateway(gateway),
            repository,
            clock,
            Arc::clone(&publisher),
        )?));
        let (sender, receiver) = mpsc::channel();
        let last_snapshot = Arc::new(Mutex::new(None));
        let controller = Arc::new(Self {
            switcher: Arc::clone(&switcher),
            monitor: Mutex::new(monitor),
            sender,
            worker: Mutex::new(None),
            monitoring: AtomicBool::new(false),
            debug_capture: AtomicBool::new(false),
            last_snapshot: Arc::clone(&last_snapshot),
            quick_set_session: Mutex::new(None),
            publisher: Arc::clone(&publisher),
            ui_locale: Arc::clone(&ui_locale),
            app_display_names,
        });
        let worker = thread::Builder::new()
            .name("readytype-switcher".to_owned())
            .spawn(move || run_worker(receiver, switcher, last_snapshot, publisher, ui_locale))
            .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
        *controller.lock_worker()? = Some(worker);
        Ok(controller)
    }

    /// Resolved display language (`ja` | `en`) for tray / UI sync.
    pub fn resolved_ui_locale(&self) -> String {
        self.ui_locale.load()
    }

    /// Re-resolve when preference is `system` (e.g. on window focus).
    pub fn refresh_resolved_ui_locale(&self) -> Result<Option<SwitcherState>, ControllerError> {
        let (preference, state) = {
            let switcher = self.lock_switcher()?;
            (
                switcher.settings().ui_locale.clone(),
                switcher.state().clone(),
            )
        };
        if SwitcherSettings::normalize_ui_locale(&preference) != "system" {
            return Ok(None);
        }
        let resolved = SwitcherSettings::resolve_ui_locale(&preference);
        if resolved == self.ui_locale.load() {
            return Ok(None);
        }
        self.ui_locale.store(&resolved);
        self.publisher.publish(&state, &resolved);
        Ok(Some(state))
    }

    pub fn state(&self) -> Result<SwitcherState, ControllerError> {
        Ok(self.lock_switcher()?.state().clone())
    }

    pub fn settings(&self) -> Result<SwitcherSettings, ControllerError> {
        Ok(self.lock_switcher()?.settings().clone())
    }

    pub fn permission_status(&self) -> Result<AccessibilityPermission, ControllerError> {
        Ok(self.lock_monitor()?.permission_status())
    }

    pub fn is_monitoring(&self) -> bool {
        self.monitoring.load(Ordering::Acquire)
    }

    pub fn is_debug_capture_enabled(&self) -> bool {
        self.debug_capture.load(Ordering::Acquire)
    }

    pub fn refresh_permission(
        self: &Arc<Self>,
    ) -> Result<AccessibilityPermission, ControllerError> {
        let permission = self.permission_status()?;
        match permission {
            AccessibilityPermission::Granted if !self.is_monitoring() => self.start_monitoring()?,
            AccessibilityPermission::Denied if self.is_monitoring() => self.stop_monitoring()?,
            AccessibilityPermission::Granted
            | AccessibilityPermission::Denied
            | AccessibilityPermission::Unsupported => {}
        }
        Ok(permission)
    }

    pub fn request_permission(
        self: &Arc<Self>,
    ) -> Result<AccessibilityPermission, ControllerError> {
        let permission = self.lock_monitor()?.request_permission()?;
        if permission == AccessibilityPermission::Granted && !self.is_monitoring() {
            self.start_monitoring()?;
        }
        Ok(permission)
    }

    pub fn open_accessibility_settings(&self) -> Result<(), ControllerError> {
        self.lock_monitor()?.open_settings()?;
        Ok(())
    }

    pub fn start_monitoring(self: &Arc<Self>) -> Result<(), ControllerError> {
        if self.is_monitoring() {
            return Ok(());
        }
        let attributes = required_attributes(self.lock_switcher()?.rules());
        let sender = self.sender.clone();
        self.lock_monitor()?.start(
            attributes,
            Arc::new(move |snapshot| {
                let _ = sender.send(WorkerMessage::Focus(snapshot));
            }),
        )?;
        self.monitoring.store(true, Ordering::Release);
        Ok(())
    }

    pub fn stop_monitoring(&self) -> Result<(), ControllerError> {
        self.lock_monitor()?.stop()?;
        self.monitoring.store(false, Ordering::Release);
        Ok(())
    }

    pub fn set_enabled(&self, enabled: bool) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.set_enabled(enabled)?;
            Ok(switcher.state().clone())
        })
    }

    pub fn set_snooze_minutes(&self, minutes: u64) -> Result<SwitcherState, ControllerError> {
        let state = self.mutate_switcher(|switcher| {
            switcher.snooze_for_minutes(minutes)?;
            Ok(switcher.state().clone())
        })?;
        self.wake_worker()?;
        Ok(state)
    }

    pub fn set_snooze_indefinitely(&self) -> Result<SwitcherState, ControllerError> {
        let state = self.mutate_switcher(|switcher| {
            switcher.snooze_indefinitely()?;
            Ok(switcher.state().clone())
        })?;
        self.wake_worker()?;
        Ok(state)
    }

    pub fn resume(&self) -> Result<SwitcherState, ControllerError> {
        let state = self.mutate_switcher(|switcher| {
            switcher.set_snoozed_until(None)?;
            Ok(switcher.state().clone())
        })?;
        self.wake_worker()?;
        Ok(state)
    }

    pub fn list_input_sources(
        &self,
        refresh: bool,
    ) -> Result<Vec<AvailableInputSource>, ControllerError> {
        let gateway = self.lock_switcher()?.gateway_handle();
        let mut gateway = gateway
            .lock()
            .map_err(|_| ControllerError::InvalidRequest("gateway lock poisoned".to_owned()))?;
        Ok(gateway.list(refresh).map_err(ApplicationError::from)?)
    }

    pub fn rules(&self) -> Result<Vec<FocusRule>, ControllerError> {
        Ok(self.lock_switcher()?.rules().to_vec())
    }

    pub fn create_rule(
        self: &Arc<Self>,
        rule: FocusRule,
    ) -> Result<SwitcherState, ControllerError> {
        let mut rules = self.rules()?;
        rules.push(rule);
        self.replace_rules(rules)
    }

    pub fn update_rule(
        self: &Arc<Self>,
        rule: FocusRule,
    ) -> Result<SwitcherState, ControllerError> {
        let mut rules = self.rules()?;
        let target = rules
            .iter_mut()
            .find(|candidate| candidate.id() == rule.id())
            .ok_or_else(|| ControllerError::InvalidRequest("rule not found".to_owned()))?;
        *target = rule;
        self.replace_rules(rules)
    }

    pub fn delete_rule(self: &Arc<Self>, id: &str) -> Result<SwitcherState, ControllerError> {
        let mut rules = self.rules()?;
        let previous_len = rules.len();
        rules.retain(|rule| rule.id() != id);
        if rules.len() == previous_len {
            return Err(ControllerError::InvalidRequest("rule not found".to_owned()));
        }
        self.replace_rules(rules)
    }

    pub fn reorder_rules(
        self: &Arc<Self>,
        ids: &[String],
    ) -> Result<SwitcherState, ControllerError> {
        let rules = self.rules()?;
        if ids.len() != rules.len() || ids.iter().collect::<HashSet<_>>().len() != ids.len() {
            return Err(ControllerError::InvalidRequest(
                "rule order must contain every rule exactly once".to_owned(),
            ));
        }
        let mut by_id: HashMap<String, FocusRule> = rules
            .into_iter()
            .map(|rule| (rule.id().to_owned(), rule))
            .collect();
        let reordered = ids
            .iter()
            .enumerate()
            .map(|(priority, id)| {
                let rule = by_id
                    .remove(id)
                    .ok_or_else(|| ControllerError::InvalidRequest("unknown rule id".to_owned()))?;
                with_priority(rule, u32::try_from(priority).unwrap_or(u32::MAX))
            })
            .collect::<Result<Vec<_>, _>>()?;
        if !by_id.is_empty() {
            return Err(ControllerError::InvalidRequest(
                "rule order omitted an existing rule".to_owned(),
            ));
        }
        self.replace_rules(reordered)
    }

    pub fn reset_default_rules(self: &Arc<Self>) -> Result<SwitcherState, ControllerError> {
        self.replace_rules(default_rules())
    }

    pub fn set_debug_capture(&self, enabled: bool) -> Result<SwitcherState, ControllerError> {
        self.lock_monitor()?.set_debug_capture(enabled)?;
        self.debug_capture.store(enabled, Ordering::Release);
        self.state()
    }

    pub fn set_debug_logging(&self, enabled: bool) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.set_debug_logging(enabled)?;
            Ok(switcher.state().clone())
        })
    }

    pub fn set_ui_locale(&self, locale: &str) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.set_ui_locale(locale)?;
            Ok(switcher.state().clone())
        })
    }

    pub fn focus_snapshot(&self) -> Result<Option<FocusSnapshot>, ControllerError> {
        Ok(self.lock_last_snapshot()?.clone())
    }

    /// Capture current focus into a Quick Set session (call before showing the popover window).
    pub fn prepare_quick_set_session(self: &Arc<Self>) -> Result<QuickSetSession, ControllerError> {
        let snapshot = self
            .lock_last_snapshot()?
            .clone()
            .ok_or_else(|| ControllerError::InvalidRequest("no focused input field".to_owned()))?;
        if snapshot.elements().is_empty() {
            return Err(ControllerError::InvalidRequest(
                "no focused input field".to_owned(),
            ));
        }
        let rules = self.rules()?;
        let settings = self.settings()?;
        let sources = self.list_input_sources(false)?;
        let app_display_name = self.app_display_names.resolve(snapshot.app().app_id());
        let session =
            build_quick_set_session(&snapshot, &rules, &settings, &sources, app_display_name);
        *self.lock_quick_set_session()? = Some(session.clone());
        Ok(session)
    }

    pub fn quick_set_session(&self) -> Result<Option<QuickSetSession>, ControllerError> {
        Ok(self.lock_quick_set_session()?.clone())
    }

    pub fn clear_quick_set_session(&self) -> Result<(), ControllerError> {
        *self.lock_quick_set_session()? = None;
        Ok(())
    }

    pub fn apply_quick_set_language(
        self: &Arc<Self>,
        language: &str,
        name: &str,
    ) -> Result<SwitcherState, ControllerError> {
        let tag = LanguageTag::parse(language)
            .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
        let session = self.lock_quick_set_session()?.clone().ok_or_else(|| {
            ControllerError::InvalidRequest("quick set session expired".to_owned())
        })?;
        let option = session
            .languages
            .iter()
            .find(|option| option.language == tag)
            .ok_or_else(|| {
                ControllerError::InvalidRequest(
                    "language is not available for quick set".to_owned(),
                )
            })?;
        let source_id = InputSourceId::parse(&option.preferred_source_id)
            .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
        let rule_name = resolve_quick_set_name(&session, name);

        let next_rules = if let Some(rule_id) = session.matched_rule_id.as_deref() {
            let mut rules = self.rules()?;
            let index = rules
                .iter()
                .position(|rule| rule.id() == rule_id)
                .ok_or_else(|| ControllerError::InvalidRequest("rule not found".to_owned()))?;
            let updated = build_updated_rule(&rules[index], tag.clone(), &rule_name)
                .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
            rules[index] = updated;
            rules
        } else {
            let mut rules = self.rules()?;
            let priority = next_quick_set_priority(&rules, &session.app_id);
            let rule_id = format!(
                "quick-set-{}-{}",
                sanitize_id_part(&session.app_id),
                self.lock_switcher()?.state().version
            );
            let created = build_created_rule(&session, tag.clone(), priority, &rule_id, &rule_name)
                .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
            rules.push(created);
            rules
        };

        let state = self.mutate_switcher(|switcher| {
            switcher.replace_rules_with_language_preference(next_rules, &tag, source_id)?;
            Ok(switcher.state().clone())
        })?;
        if self.is_monitoring() {
            self.stop_monitoring()?;
            self.start_monitoring()?;
        }
        self.wake_worker()?;
        *self.lock_quick_set_session()? = None;
        Ok(state)
    }

    pub fn logs(&self) -> Result<Vec<SwitchLogEntry>, ControllerError> {
        Ok(self.lock_switcher()?.state().logs.clone())
    }

    pub fn clear_logs(&self) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.clear_logs();
            Ok(switcher.state().clone())
        })
    }

    fn replace_rules(
        self: &Arc<Self>,
        rules: Vec<FocusRule>,
    ) -> Result<SwitcherState, ControllerError> {
        let state = self.mutate_switcher(|switcher| {
            switcher.replace_rules(rules)?;
            Ok(switcher.state().clone())
        })?;
        if self.is_monitoring() {
            self.stop_monitoring()?;
            self.start_monitoring()?;
        }
        self.wake_worker()?;
        Ok(state)
    }

    /// Mutate under the switcher lock, then publish only after the guard drops.
    fn mutate_switcher<T>(
        &self,
        f: impl FnOnce(&mut RuntimeSwitcher) -> Result<T, ControllerError>,
    ) -> Result<T, ControllerError> {
        let (result, pending) = {
            let mut guard = self.lock_switcher()?;
            let result = f(&mut guard)?;
            let pending = guard.drain_publish_snapshot();
            (result, pending)
        };
        self.dispatch_publish(pending);
        Ok(result)
    }

    fn dispatch_publish(&self, pending: Option<(SwitcherState, String)>) {
        if let Some((state, preference)) = pending {
            let resolved = SwitcherSettings::resolve_ui_locale(&preference);
            self.ui_locale.store(&resolved);
            self.publisher.publish(&state, &resolved);
        }
    }

    fn wake_worker(&self) -> Result<(), ControllerError> {
        self.sender
            .send(WorkerMessage::SettingsChanged)
            .map_err(|_| ControllerError::WorkerStopped)
    }

    fn lock_switcher(&self) -> Result<MutexGuard<'_, RuntimeSwitcher>, ControllerError> {
        self.switcher
            .lock()
            .map_err(|_| ControllerError::WorkerStopped)
    }

    fn lock_monitor(&self) -> Result<MutexGuard<'_, Box<dyn FocusMonitor>>, ControllerError> {
        self.monitor
            .lock()
            .map_err(|_| ControllerError::WorkerStopped)
    }

    fn lock_worker(&self) -> Result<MutexGuard<'_, Option<JoinHandle<()>>>, ControllerError> {
        self.worker
            .lock()
            .map_err(|_| ControllerError::WorkerStopped)
    }

    fn lock_last_snapshot(&self) -> Result<MutexGuard<'_, Option<FocusSnapshot>>, ControllerError> {
        self.last_snapshot
            .lock()
            .map_err(|_| ControllerError::WorkerStopped)
    }

    fn lock_quick_set_session(
        &self,
    ) -> Result<MutexGuard<'_, Option<QuickSetSession>>, ControllerError> {
        self.quick_set_session
            .lock()
            .map_err(|_| ControllerError::WorkerStopped)
    }
}

impl Drop for SwitcherController {
    fn drop(&mut self) {
        let _ = self.monitor.get_mut().map(|monitor| monitor.stop());
        let _ = self.sender.send(WorkerMessage::Stop);
        if let Ok(worker) = self.worker.get_mut() {
            if let Some(worker) = worker.take() {
                let _ = worker.join();
            }
        }
    }
}

fn repository_gateway(
    gateway: Box<dyn InputSourceGateway>,
) -> Arc<Mutex<Box<dyn InputSourceGateway>>> {
    Arc::new(Mutex::new(gateway))
}

fn sanitize_id_part(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('-');
        }
    }
    if out.is_empty() {
        "app".to_owned()
    } else {
        out
    }
}

fn run_worker(
    receiver: Receiver<WorkerMessage>,
    switcher: Arc<Mutex<RuntimeSwitcher>>,
    last_snapshot: Arc<Mutex<Option<FocusSnapshot>>>,
    publisher: Arc<dyn SwitcherEventPublisher>,
    ui_locale: Arc<UiLocaleCache>,
) {
    let dispatch = |pending: Option<(SwitcherState, String)>| {
        if let Some((state, preference)) = pending {
            let resolved = SwitcherSettings::resolve_ui_locale(&preference);
            ui_locale.store(&resolved);
            publisher.publish(&state, &resolved);
        }
    };

    let mut active: Option<(PlatformAppId, u64)> = None;
    loop {
        let wait = switcher
            .lock()
            .ok()
            .and_then(|switcher| switcher.snooze_remaining_ms())
            .map(Duration::from_millis);
        let message = match wait {
            Some(wait) => receiver.recv_timeout(wait),
            None => receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
        };
        match message {
            Ok(WorkerMessage::Focus(snapshot)) => {
                if let Ok(mut latest) = last_snapshot.lock() {
                    *latest = Some(snapshot.clone());
                }
                let key = (snapshot.app().clone(), snapshot.generation());
                let prepared = {
                    let Ok(mut switcher) = switcher.lock() else {
                        continue;
                    };
                    if active.as_ref() != Some(&key) {
                        let _ = switcher.activate(key.0.clone(), key.1);
                        active = Some(key);
                    }
                    let gateway = switcher.gateway_handle();
                    let prepared = switcher.prepare_focus(snapshot);
                    let pending = switcher.drain_publish_snapshot();
                    (gateway, prepared, pending)
                };
                // TIS はメインキューへ sync するため、switcher ロックを握ったまま待たない。
                let (gateway, prepared, pending) = prepared;
                dispatch(pending);
                let Ok(prepared) = prepared else {
                    continue;
                };
                match prepared {
                    PreparedFocus::Done(_) => {}
                    PreparedFocus::NeedsSwitch {
                        snapshot,
                        rule,
                        target,
                    } => {
                        let switch_result = match gateway.lock() {
                            Ok(mut gateway) => switch_input_source(&mut *gateway, &target),
                            Err(_) => Err(ApplicationError::InvalidSettings(
                                "gateway lock poisoned".to_owned(),
                            )),
                        };
                        match switch_result {
                            Ok((current, select_result)) => {
                                let pending = if let Ok(mut switcher) = switcher.lock() {
                                    let _ = switcher.complete_focus(
                                        snapshot,
                                        *rule,
                                        target,
                                        current,
                                        select_result,
                                    );
                                    switcher.drain_publish_snapshot()
                                } else {
                                    None
                                };
                                dispatch(pending);
                            }
                            Err(error) => {
                                let pending = if let Ok(mut switcher) = switcher.lock() {
                                    switcher.record_switch_pipeline_error(
                                        &snapshot,
                                        rule.id(),
                                        &target,
                                        &error,
                                    );
                                    switcher.drain_publish_snapshot()
                                } else {
                                    None
                                };
                                dispatch(pending);
                            }
                        }
                    }
                }
            }
            Ok(WorkerMessage::SettingsChanged) => {}
            Ok(WorkerMessage::Stop) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {
                let pending = if let Ok(mut switcher) = switcher.lock() {
                    let _ = switcher.resume_if_due();
                    switcher.drain_publish_snapshot()
                } else {
                    None
                };
                dispatch(pending);
            }
        }
    }
}

fn required_attributes(rules: &[FocusRule]) -> Vec<AxAttribute> {
    let mut seen = HashSet::new();
    let mut attributes = Vec::new();
    // Always read AccName / placeholder sources so Quick Set can draft a label.
    for attribute in [
        AxAttribute::Role,
        AxAttribute::Title,
        AxAttribute::Description,
        AxAttribute::RoleDescription,
        AxAttribute::Placeholder,
        AxAttribute::Identifier,
        AxAttribute::DomIdentifier,
        AxAttribute::DomClassList,
    ] {
        if seen.insert(attribute) {
            attributes.push(attribute);
        }
    }
    for attribute in rules
        .iter()
        .filter(|rule| rule.enabled())
        .flat_map(|rule| rule.conditions())
        .map(|condition| condition.attribute())
    {
        if seen.insert(attribute) {
            attributes.push(attribute);
        }
    }
    attributes
}

fn with_priority(rule: FocusRule, priority: u32) -> Result<FocusRule, ControllerError> {
    FocusRule::new(
        rule.id(),
        rule.name(),
        rule.enabled(),
        rule.platform(),
        rule.app_id(),
        priority,
        rule.conditions().to_vec(),
        rule.target().clone(),
    )
    .map_err(|error| ControllerError::InvalidRequest(error.to_string()))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc,
    };

    use super::SwitcherController;
    use crate::{
        application::{
            ports::{
                app_display_name::AppDisplayNameResolver,
                clock::Clock,
                focus_monitor::{
                    AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
                },
                input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
                settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
                switcher_events::SwitcherEventPublisher,
            },
            settings::SwitcherSettings,
            switcher_state::SwitcherState,
        },
        domain::input_source::{AxAttribute, InputSourceId},
    };

    struct FakeGateway;

    impl InputSourceGateway for FakeGateway {
        fn list(&mut self, _refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError> {
            Ok(Vec::new())
        }

        fn current(&self) -> Result<InputSourceId, GatewayError> {
            InputSourceId::parse("en").map_err(|error| GatewayError::Operation(error.to_string()))
        }

        fn select(&mut self, _id: &InputSourceId) -> Result<(), GatewayError> {
            Ok(())
        }
    }

    struct FakeRepository;

    impl SettingsRepository for FakeRepository {
        fn load(&self) -> Result<LoadOutcome, SettingsError> {
            Ok(LoadOutcome::Defaults {
                settings: SwitcherSettings::defaults(),
            })
        }

        fn save(&self, _settings: &SwitcherSettings) -> Result<(), SettingsError> {
            Ok(())
        }
    }

    struct FakeClock(AtomicU64);

    impl Clock for FakeClock {
        fn now_unix_ms(&self) -> u64 {
            self.0.load(Ordering::Acquire)
        }
    }

    struct NoopPublisher;

    impl SwitcherEventPublisher for NoopPublisher {
        fn publish(&self, _state: &SwitcherState, _locale: &str) {}
    }

    #[derive(Default)]
    struct MonitorData {
        trusted: AtomicBool,
        starts: AtomicUsize,
        stops: AtomicUsize,
    }

    struct FakeMonitor(Arc<MonitorData>);

    impl FocusMonitor for FakeMonitor {
        fn start(
            &mut self,
            _attributes: Vec<AxAttribute>,
            _sink: FocusEventSink,
        ) -> Result<(), FocusMonitorError> {
            self.0.starts.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }

        fn stop(&mut self) -> Result<(), FocusMonitorError> {
            self.0.stops.fetch_add(1, Ordering::AcqRel);
            Ok(())
        }

        fn set_debug_capture(&mut self, _enabled: bool) -> Result<(), FocusMonitorError> {
            Ok(())
        }

        fn permission_status(&self) -> AccessibilityPermission {
            if self.0.trusted.load(Ordering::Acquire) {
                AccessibilityPermission::Granted
            } else {
                AccessibilityPermission::Denied
            }
        }

        fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError> {
            Ok(self.permission_status())
        }

        fn open_settings(&self) -> Result<(), FocusMonitorError> {
            Ok(())
        }
    }

    struct NoopDisplayNames;

    impl AppDisplayNameResolver for NoopDisplayNames {
        fn resolve(&self, _bundle_id: &str) -> Option<String> {
            None
        }
    }

    fn controller(monitor: Arc<MonitorData>) -> Arc<SwitcherController> {
        SwitcherController::new(
            SwitcherSettings::defaults(),
            Box::new(FakeGateway),
            Arc::new(FakeRepository),
            Arc::new(FakeClock(AtomicU64::new(1_000))),
            Arc::new(NoopPublisher),
            Box::new(FakeMonitor(monitor)),
            Arc::new(NoopDisplayNames),
        )
        .expect("controller")
    }

    #[test]
    fn permission_restoration_restarts_monitoring_without_process_restart() {
        let monitor = Arc::new(MonitorData::default());
        let controller = controller(Arc::clone(&monitor));
        controller.refresh_permission().expect("denied state");
        monitor.trusted.store(true, Ordering::Release);

        controller.refresh_permission().expect("restart");

        assert_eq!(monitor.starts.load(Ordering::Acquire), 1);
        assert!(controller.is_monitoring());
    }

    #[test]
    fn permission_revocation_stops_monitoring_without_terminating_controller() {
        let monitor = Arc::new(MonitorData::default());
        monitor.trusted.store(true, Ordering::Release);
        let controller = controller(Arc::clone(&monitor));
        controller.refresh_permission().expect("start");
        monitor.trusted.store(false, Ordering::Release);

        controller.refresh_permission().expect("stop");

        assert_eq!(monitor.stops.load(Ordering::Acquire), 1);
        assert!(!controller.is_monitoring());
        assert!(controller.state().is_ok());
    }

    #[test]
    fn rules_can_be_deleted_reordered_and_reset() {
        let controller = controller(Arc::new(MonitorData::default()));
        let first_id = controller.rules().expect("rules")[0].id().to_owned();
        controller.delete_rule(&first_id).expect("delete");
        let mut ids: Vec<_> = controller
            .rules()
            .expect("rules")
            .iter()
            .map(|rule| rule.id().to_owned())
            .collect();
        ids.reverse();
        controller.reorder_rules(&ids).expect("reorder");
        assert_eq!(controller.rules().expect("rules")[0].id(), ids[0]);

        controller.reset_default_rules().expect("reset");
        assert!(controller
            .rules()
            .expect("rules")
            .iter()
            .any(|rule| rule.id() == first_id));
    }

    #[test]
    fn debug_options_and_duplicate_rules_are_validated() {
        let controller = controller(Arc::new(MonitorData::default()));
        controller.set_debug_logging(true).expect("debug logging");
        controller.set_debug_capture(true).expect("capture");
        let settings = controller.settings().expect("settings");
        assert!(settings.debug_logging);
        assert!(controller.is_debug_capture_enabled());

        let duplicate = controller.rules().expect("rules")[0].clone();
        assert!(controller.create_rule(duplicate).is_err());
        controller.clear_logs().expect("clear logs");
        assert!(controller.logs().expect("logs").is_empty());
    }

    #[test]
    fn zero_minute_snooze_expires_without_a_focus_event() {
        let controller = controller(Arc::new(MonitorData::default()));
        controller.set_snooze_minutes(0).expect("snooze");

        for _ in 0..100 {
            if controller
                .state()
                .expect("state")
                .snoozed_until_unix_ms
                .is_none()
            {
                return;
            }
            std::thread::yield_now();
        }
        panic!("snooze did not expire");
    }
}
