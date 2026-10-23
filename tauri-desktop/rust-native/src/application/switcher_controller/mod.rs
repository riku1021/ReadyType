use std::{
    collections::{HashMap, HashSet},
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, RecvTimeoutError, Sender},
        Arc, Mutex, MutexGuard,
    },
    thread::{self, JoinHandle},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::domain::input_source::{
    readytype_lab_switch_rules, AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot,
    InputSourceId, Platform, PlatformAppId, RuleTarget, READYTYPE_BUNDLE_ID,
};

use super::{
    input_source_switcher::{
        switch_input_source, ApplicationError, CompleteFocusInput, InputSourceSwitcher,
        PreparedFocus,
    },
    ports::{
        app_display_name::AppDisplayNameResolver,
        clock::Clock,
        current_input_source_change::CurrentInputSourceChangeMonitor,
        diagnostic_log::SharedDiagnosticLogSink,
        focus_monitor::{AccessibilityPermission, FocusMonitor, FocusMonitorError},
        input_source::{AvailableInputSource, InputSourceGateway},
        settings_repository::SettingsRepository,
        switcher_events::SwitcherEventPublisher,
    },
    quick_set::{
        build_created_rule, build_quick_set_session, build_updated_rule, next_quick_set_priority,
        resolve_quick_set_name, rule_target_from_source_id, QuickSetSession,
    },
    settings::SwitcherSettings,
    switch_proto,
    switcher_state::{SwitchLogEntry, SwitcherState},
    ui_locale_cache::UiLocaleCache,
};

pub struct SwitcherControllerDeps {
    pub settings: SwitcherSettings,
    pub gateway: Box<dyn InputSourceGateway>,
    pub repository: Arc<dyn SettingsRepository>,
    pub clock: Arc<dyn Clock>,
    pub publisher: Arc<dyn SwitcherEventPublisher>,
    pub monitor: Box<dyn FocusMonitor>,
    pub app_display_names: Arc<dyn AppDisplayNameResolver>,
    /// TIS change notifications (PROTO_TIS_NOTIFY). Composition may pass Noop.
    pub tis_change_monitor: Box<dyn CurrentInputSourceChangeMonitor>,
}

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

pub(crate) enum WorkerMessage {
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
    /// Lab ネイティブ窓の AppKit フォーカス通知用（AX 世代と衝突しにくいレンジ）。
    lab_focus_generation: AtomicU64,
    last_snapshot: Arc<Mutex<Option<FocusSnapshot>>>,
    quick_set_session: Mutex<Option<QuickSetSession>>,
    publisher: Arc<dyn SwitcherEventPublisher>,
    ui_locale: Arc<UiLocaleCache>,
    app_display_names: Arc<dyn AppDisplayNameResolver>,
    tis_change_monitor: Mutex<Box<dyn CurrentInputSourceChangeMonitor>>,
}

impl SwitcherController {
    pub fn new(deps: SwitcherControllerDeps) -> Result<Arc<Self>, ControllerError> {
        let SwitcherControllerDeps {
            settings,
            gateway,
            repository,
            clock,
            publisher,
            monitor,
            app_display_names,
            tis_change_monitor,
        } = deps;
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
            lab_focus_generation: AtomicU64::new(1_000_000),
            last_snapshot: Arc::clone(&last_snapshot),
            quick_set_session: Mutex::new(None),
            publisher: Arc::clone(&publisher),
            ui_locale: Arc::clone(&ui_locale),
            app_display_names,
            tis_change_monitor: Mutex::new(tis_change_monitor),
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

    /// Composition root から follow-up reselect の診断 sink を配線する。
    pub fn bind_followup_reselect_log<F>(&self, bind: F)
    where
        F: FnOnce(Arc<dyn Fn(u64, i32, Option<InputSourceId>) + Send + Sync>),
    {
        let switcher = Arc::clone(&self.switcher);
        bind(Arc::new(move |delay_ms, status, current| {
            if let Ok(mut switcher) = switcher.lock() {
                switcher.note_followup_reselect(delay_ms, status, current.as_ref());
            }
        }));
    }

    /// Composition root から診断ファイル等の sink を配線する。
    pub fn set_diagnostic_log_sink(&self, sink: Option<SharedDiagnosticLogSink>) {
        if let Ok(mut switcher) = self.switcher.lock() {
            switcher.set_diagnostic_log_sink(sink);
        }
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
        self.refresh_localized_sources()?;
        self.publisher.publish(&state, &resolved);
        self.publisher.publish_resolved_ui_locale(&resolved);
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
        {
            let switcher = self.lock_switcher()?;
            switcher.warm_current_cache();
        }
        self.start_tis_change_monitor()?;
        self.monitoring.store(true, Ordering::Release);
        Ok(())
    }

    pub fn stop_monitoring(&self) -> Result<(), ControllerError> {
        self.stop_tis_change_monitor()?;
        self.lock_monitor()?.stop()?;
        self.monitoring.store(false, Ordering::Release);
        Ok(())
    }

    fn start_tis_change_monitor(self: &Arc<Self>) -> Result<(), ControllerError> {
        if !switch_proto::tis_notify() {
            return Ok(());
        }
        let switcher = Arc::clone(&self.switcher);
        let sink: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            let Ok(mut guard) = switcher.lock() else {
                return;
            };
            guard.invalidate_current_cache();
            if switch_proto::manual_override() {
                let gateway = guard.gateway_handle();
                let current = gateway
                    .lock()
                    .ok()
                    .and_then(|gateway| gateway.current().ok());
                if let Some(current) = current {
                    guard.note_external_input_source_change(&current);
                }
            }
        });
        let mut monitor = self
            .tis_change_monitor
            .lock()
            .map_err(|_| ControllerError::InvalidRequest("tis monitor lock poisoned".to_owned()))?;
        monitor
            .start(sink)
            .map_err(ControllerError::InvalidRequest)?;
        Ok(())
    }

    fn stop_tis_change_monitor(&self) -> Result<(), ControllerError> {
        let mut monitor = self
            .tis_change_monitor
            .lock()
            .map_err(|_| ControllerError::InvalidRequest("tis monitor lock poisoned".to_owned()))?;
        monitor.stop().map_err(ControllerError::InvalidRequest)?;
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
        let locale = self.ui_locale.load();
        let gateway = self.lock_switcher()?.gateway_handle();
        let sources = {
            let mut gateway = gateway
                .lock()
                .map_err(|_| ControllerError::InvalidRequest("gateway lock poisoned".to_owned()))?;
            gateway
                .list(refresh, &locale)
                .map_err(ApplicationError::from)?
        };
        // gateway ロックを解放してから switcher を取る（他経路は switcher → gateway の順）。
        // 表示名の保存失敗で一覧取得を失敗させない。
        let _ = self
            .lock_switcher()?
            .record_input_source_display_names(&sources);
        Ok(sources)
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

    /// ルール一覧を空にする（IPC 名 `reset_default_rules` は互換のため維持）。
    pub fn reset_default_rules(self: &Arc<Self>) -> Result<SwitcherState, ControllerError> {
        self.replace_rules(Vec::new())
    }

    /// 既存ルールを保ったまま Lab 切替テスト用プリセットを upsert する。
    /// 件数だけでなく条件内容が変わったときも置換する（例: DomIdentifier → Identifier）。
    pub fn ensure_lab_switch_rules(self: &Arc<Self>) -> Result<SwitcherState, ControllerError> {
        let lab = readytype_lab_switch_rules();
        let lab_ids = lab
            .iter()
            .map(|rule| rule.id().to_owned())
            .collect::<HashSet<_>>();
        let current = self.rules()?;
        let mut next = current.clone();
        next.retain(|rule| !lab_ids.contains(rule.id()));
        next.extend(lab);
        if next == current {
            return self.state();
        }
        self.replace_rules(next)
    }

    pub fn set_debug_capture(&self, enabled: bool) -> Result<SwitcherState, ControllerError> {
        self.lock_monitor()?.set_debug_capture(enabled)?;
        self.debug_capture.store(enabled, Ordering::Release);
        self.state()
    }

    pub fn set_ui_locale(&self, locale: &str) -> Result<SwitcherState, ControllerError> {
        let state = self.mutate_switcher(|switcher| {
            switcher.set_ui_locale(locale)?;
            Ok(switcher.state().clone())
        })?;
        self.refresh_localized_sources()?;
        Ok(state)
    }

    pub fn set_quick_set_shortcut(&self, shortcut: &str) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.set_quick_set_shortcut(shortcut)?;
            Ok(switcher.state().clone())
        })
    }

    pub fn complete_onboarding(&self) -> Result<SwitcherState, ControllerError> {
        self.mutate_switcher(|switcher| {
            switcher.complete_onboarding()?;
            Ok(switcher.state().clone())
        })
    }

    pub fn focus_snapshot(&self) -> Result<Option<FocusSnapshot>, ControllerError> {
        Ok(self.lock_last_snapshot()?.clone())
    }

    /// Lab ネイティブ入力欄フォーカス（同一プロセス AX が空のときのフォールバック）。
    ///
    /// 本番と同じ `WorkerMessage::Focus` → ルール照合 → 切替経路を使う。
    pub fn report_lab_focus(&self, identifier: &str) -> Result<(), ControllerError> {
        let identifier = identifier.trim();
        if identifier.is_empty() {
            return Err(ControllerError::InvalidRequest(
                "lab identifier is required".to_owned(),
            ));
        }
        if !self.is_monitoring() {
            return Err(ControllerError::InvalidRequest(
                "focus monitoring is not running".to_owned(),
            ));
        }
        let pid = i32::try_from(std::process::id()).unwrap_or(0);
        let app = PlatformAppId::new(Platform::MacOs, READYTYPE_BUNDLE_ID, pid)
            .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
        let generation = self.lab_focus_generation.fetch_add(1, Ordering::AcqRel);
        let focused = AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextField")
            .with(AxAttribute::Identifier, identifier);
        let snapshot = FocusSnapshot::new(app, generation, vec![focused]);
        self.sender
            .send(WorkerMessage::Focus(snapshot))
            .map_err(|_| ControllerError::WorkerStopped)?;
        Ok(())
    }

    /// Capture current focus into a Quick Set session (call before showing the popover window).
    pub fn prepare_quick_set_session(self: &Arc<Self>) -> Result<QuickSetSession, ControllerError> {
        let snapshot = self.lock_last_snapshot()?.clone();
        let result = self.build_quick_set_session(snapshot.as_ref());
        let outcome = match &result {
            Ok(_) => "ready".to_owned(),
            Err(ControllerError::InvalidRequest(message)) => message.clone(),
            Err(error) => error.to_string(),
        };
        self.record_quick_set_launch(&outcome, snapshot.as_ref());
        result
    }

    fn build_quick_set_session(
        self: &Arc<Self>,
        snapshot: Option<&FocusSnapshot>,
    ) -> Result<QuickSetSession, ControllerError> {
        let Some(snapshot) = snapshot else {
            return Err(ControllerError::InvalidRequest(
                "no focused input field".to_owned(),
            ));
        };
        if snapshot.elements().is_empty() {
            return Err(ControllerError::InvalidRequest(
                "no focused input field".to_owned(),
            ));
        }
        // ReadyType 前面時に他アプリの古い last_snapshot で Quick Set しない。
        let mismatch = {
            let monitor = self.lock_monitor()?;
            monitor.frontmost_app().and_then(|frontmost| {
                (frontmost.app_id() != snapshot.app().app_id()).then(|| {
                    format!(
                        "focus snapshot is for {} but frontmost is {}; focus a field in the frontmost app first",
                        snapshot.app().app_id(),
                        frontmost.app_id()
                    )
                })
            })
        };
        if let Some(message) = mismatch {
            return Err(ControllerError::InvalidRequest(message));
        }
        let rules = self.rules()?;
        let settings = self.settings()?;
        // Match app-settings LanguageSelect: always refresh OS input sources.
        let sources = self.list_input_sources(true)?;
        let app_display_name = self.app_display_names.resolve(snapshot.app().app_id());
        let session =
            build_quick_set_session(snapshot, &rules, &settings, &sources, app_display_name);
        *self.lock_quick_set_session()? = Some(session.clone());
        Ok(session)
    }

    fn record_quick_set_launch(&self, outcome: &str, snapshot: Option<&FocusSnapshot>) {
        let shortcut = self
            .settings()
            .map(|settings| settings.quick_set_shortcut)
            .unwrap_or_default();
        let Ok(switcher) = self.lock_switcher() else {
            return;
        };
        switcher.note_quick_set_launch(&shortcut, outcome, snapshot);
    }

    pub fn quick_set_session(&self) -> Result<Option<QuickSetSession>, ControllerError> {
        Ok(self.lock_quick_set_session()?.clone())
    }

    pub fn clear_quick_set_session(&self) -> Result<(), ControllerError> {
        *self.lock_quick_set_session()? = None;
        Ok(())
    }

    pub fn apply_quick_set_source(
        self: &Arc<Self>,
        source_id: &str,
        name: &str,
    ) -> Result<SwitcherState, ControllerError> {
        let source_id = InputSourceId::parse(source_id)
            .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
        let session = self.lock_quick_set_session()?.clone().ok_or_else(|| {
            ControllerError::InvalidRequest("quick set session expired".to_owned())
        })?;
        let allowed = session.sources.iter().any(|source| source.id == source_id)
            || session
                .language_preferences
                .values()
                .any(|preferred| preferred.as_ref() == Some(&source_id));
        if !allowed {
            return Err(ControllerError::InvalidRequest(
                "input source is not available for quick set".to_owned(),
            ));
        }
        let settings = self.settings()?;
        let target = rule_target_from_source_id(&source_id, &settings);
        let rule_name = resolve_quick_set_name(&session, name);

        let next_rules = if let Some(rule_id) = session.matched_rule_id.as_deref() {
            let mut rules = self.rules()?;
            let index = rules
                .iter()
                .position(|rule| rule.id() == rule_id)
                .ok_or_else(|| ControllerError::InvalidRequest("rule not found".to_owned()))?;
            let updated = build_updated_rule(&rules[index], target.clone(), &rule_name)
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
            let created =
                build_created_rule(&session, target.clone(), priority, &rule_id, &rule_name)
                    .map_err(|error| ControllerError::InvalidRequest(error.to_string()))?;
            rules.push(created);
            rules
        };

        let state = self.mutate_switcher(|switcher| {
            match &target {
                RuleTarget::PreferredLanguage(tag) => {
                    switcher.replace_rules_with_language_preference(
                        next_rules,
                        tag,
                        source_id.clone(),
                    )?;
                }
                RuleTarget::Exact(_) => {
                    switcher.replace_rules(next_rules)?;
                }
            }
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

    /// Refresh input-source display names for the current UI locale.
    fn refresh_localized_sources(&self) -> Result<(), ControllerError> {
        let _ = self.list_input_sources(true)?;
        Ok(())
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
    let mut deferred: Option<WorkerMessage> = None;
    loop {
        let message = if let Some(pending) = deferred.take() {
            Ok(pending)
        } else {
            let wait = switcher
                .lock()
                .ok()
                .and_then(|switcher| switcher.snooze_remaining_ms())
                .map(Duration::from_millis);
            match wait {
                Some(wait) => receiver.recv_timeout(wait),
                None => receiver.recv().map_err(|_| RecvTimeoutError::Disconnected),
            }
        };
        match message {
            Ok(WorkerMessage::Focus(snapshot)) => {
                let (snapshot, next_deferred) = if switch_proto::drain_focus() {
                    drain_queued_focus(&receiver, snapshot)
                } else {
                    (snapshot, None)
                };
                deferred = next_deferred;
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
                    let current_cache = switcher.current_cache_handle();
                    let prepared = switcher.prepare_focus(snapshot);
                    let pending = switcher.drain_publish_snapshot();
                    (gateway, current_cache, prepared, pending)
                };
                // TIS はメインキューへ sync するため、switcher ロックを握ったまま待たない。
                let (gateway, current_cache, prepared, pending) = prepared;
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
                        prepare_micros,
                    } => {
                        if switch_proto::manual_override() {
                            if let Ok(mut switcher) = switcher.lock() {
                                switcher.set_suppress_tis_as_manual(true);
                            }
                        }
                        let locale = ui_locale.load();
                        let switch_result = match gateway.lock() {
                            Ok(mut gateway) => switch_input_source(
                                &mut *gateway,
                                &target,
                                &locale,
                                Some(&current_cache),
                            ),
                            Err(_) => Err(ApplicationError::InvalidSettings(
                                "gateway lock poisoned".to_owned(),
                            )),
                        };
                        match switch_result {
                            Ok((current, select_result, source_timing)) => {
                                let pending = if let Ok(mut switcher) = switcher.lock() {
                                    let _ = switcher.complete_focus(CompleteFocusInput {
                                        snapshot,
                                        rule: *rule,
                                        target,
                                        current,
                                        select_result,
                                        prepare_micros,
                                        source_timing,
                                    });
                                    if switch_proto::manual_override() {
                                        switcher.set_suppress_tis_as_manual(false);
                                    }
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
                                    if switch_proto::manual_override() {
                                        switcher.set_suppress_tis_as_manual(false);
                                    }
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

/// Keep the latest Focus; defer the first non-Focus message (PROTO_DRAIN_FOCUS).
pub(crate) fn drain_queued_focus(
    receiver: &Receiver<WorkerMessage>,
    first: FocusSnapshot,
) -> (FocusSnapshot, Option<WorkerMessage>) {
    let mut latest = first;
    loop {
        match receiver.try_recv() {
            Ok(WorkerMessage::Focus(next)) => latest = next,
            Ok(other) => return (latest, Some(other)),
            Err(_) => return (latest, None),
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
mod tests;
