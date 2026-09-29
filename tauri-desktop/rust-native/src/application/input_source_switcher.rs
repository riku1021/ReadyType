use std::{
    fmt,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::domain::input_source::{
    AxAttribute, FocusRule, FocusSnapshot, InputSourceId, PlatformAppId, RuleSet, RuleTarget,
};

use super::{
    ports::{
        clock::Clock,
        input_source::{GatewayError, InputSourceGateway},
        settings_repository::{SettingsError, SettingsRepository},
        switcher_events::SwitcherEventPublisher,
    },
    settings::SwitcherSettings,
    switcher_state::{SwitchLogEntry, SwitcherState},
};

const MAX_LOGS: usize = 200;

/// Far-future deadline for "pause until resume". Kept within JS `Number.MAX_SAFE_INTEGER`.
pub const INDEFINITE_SNOOZE_UNTIL_UNIX_MS: u64 = 253_402_300_799_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwitchDecision {
    Ignored,
    Disabled,
    Snoozed,
    NoRule,
    AlreadySelected,
    Switched,
    Failed,
}

#[derive(Debug)]
pub enum ApplicationError {
    InvalidSettings(String),
    Settings(SettingsError),
    Gateway(GatewayError),
}

impl fmt::Display for ApplicationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSettings(message) => write!(formatter, "invalid settings: {message}"),
            Self::Settings(error) => error.fmt(formatter),
            Self::Gateway(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ApplicationError {}

impl From<SettingsError> for ApplicationError {
    fn from(value: SettingsError) -> Self {
        Self::Settings(value)
    }
}

impl From<GatewayError> for ApplicationError {
    fn from(value: GatewayError) -> Self {
        Self::Gateway(value)
    }
}

pub struct InputSourceSwitcher<G, R, C, E> {
    settings: SwitcherSettings,
    rules: RuleSet,
    gateway: Arc<Mutex<G>>,
    repository: R,
    clock: C,
    publisher: E,
    state: SwitcherState,
    /// Set by `bump_state`; drained outside the switcher mutex before publish.
    publish_pending: bool,
    active_app: Option<PlatformAppId>,
    active_generation: u64,
    handled_generation: Option<u64>,
}

/// `prepare_focus` の結果。gateway 呼び出しはロック外で行う。
pub enum PreparedFocus {
    Done(SwitchDecision),
    NeedsSwitch {
        snapshot: FocusSnapshot,
        rule: Box<FocusRule>,
        target: InputSourceId,
    },
}

impl<G, R, C, E> InputSourceSwitcher<G, R, C, E>
where
    G: InputSourceGateway,
    R: SettingsRepository,
    C: Clock,
    E: SwitcherEventPublisher,
{
    pub fn new(
        settings: SwitcherSettings,
        gateway: Arc<Mutex<G>>,
        repository: R,
        clock: C,
        publisher: E,
    ) -> Result<Self, ApplicationError> {
        settings
            .validate()
            .map_err(|error| ApplicationError::InvalidSettings(error.to_string()))?;
        let mut settings = settings;
        if settings.language_preferences.is_empty() {
            if let Ok(mut guard) = gateway.lock() {
                if let Ok(sources) = guard.list(false) {
                    let current = guard.current().ok();
                    let seeded = super::settings::seed_language_preferences_from_sources(
                        &sources,
                        current.as_ref(),
                    );
                    if !seeded.is_empty() {
                        settings.language_preferences = seeded;
                        repository.save(&settings)?;
                    }
                }
            }
        }
        let rules = RuleSet::new(settings.rules.clone())
            .map_err(|error| ApplicationError::InvalidSettings(error.to_string()))?;
        let state = SwitcherState::new(settings.enabled, settings.snoozed_until_unix_ms);
        Ok(Self {
            settings,
            rules,
            gateway,
            repository,
            clock,
            publisher,
            state,
            publish_pending: false,
            active_app: None,
            active_generation: 0,
            handled_generation: None,
        })
    }

    pub fn gateway_handle(&self) -> Arc<Mutex<G>> {
        Arc::clone(&self.gateway)
    }

    pub fn state(&self) -> &SwitcherState {
        &self.state
    }

    pub fn rules(&self) -> &[FocusRule] {
        self.rules.rules()
    }

    pub fn settings(&self) -> &SwitcherSettings {
        &self.settings
    }

    pub fn list_input_sources(
        &mut self,
        refresh: bool,
    ) -> Result<Vec<super::ports::input_source::AvailableInputSource>, ApplicationError> {
        self.gateway
            .lock()
            .map_err(|_| ApplicationError::InvalidSettings("gateway lock poisoned".to_owned()))?
            .list(refresh)
            .map_err(ApplicationError::from)
    }

    pub fn set_enabled(&mut self, enabled: bool) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.enabled = enabled;
        self.commit_settings(candidate, None)
    }

    pub fn set_snoozed_until(
        &mut self,
        deadline_unix_ms: Option<u64>,
    ) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.snoozed_until_unix_ms = deadline_unix_ms;
        self.commit_settings(candidate, None)
    }

    pub fn snooze_for_minutes(&mut self, minutes: u64) -> Result<(), ApplicationError> {
        let deadline = self
            .clock
            .now_unix_ms()
            .saturating_add(minutes.saturating_mul(60_000));
        self.set_snoozed_until(Some(deadline))
    }

    /// Pause until the user explicitly resumes (JS-safe far-future deadline).
    pub fn snooze_indefinitely(&mut self) -> Result<(), ApplicationError> {
        self.set_snoozed_until(Some(INDEFINITE_SNOOZE_UNTIL_UNIX_MS))
    }

    pub fn resume_if_due(&mut self) -> Result<bool, ApplicationError> {
        let Some(deadline) = self.settings.snoozed_until_unix_ms else {
            return Ok(false);
        };
        if deadline > self.clock.now_unix_ms() {
            return Ok(false);
        }
        self.set_snoozed_until(None)?;
        Ok(true)
    }

    pub fn snooze_remaining_ms(&self) -> Option<u64> {
        self.settings
            .snoozed_until_unix_ms
            .map(|deadline| deadline.saturating_sub(self.clock.now_unix_ms()))
    }

    pub fn set_debug_logging(&mut self, enabled: bool) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.debug_logging = enabled;
        self.commit_settings(candidate, None)
    }

    pub fn set_ui_locale(&mut self, locale: &str) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.ui_locale = SwitcherSettings::normalize_ui_locale(locale);
        self.commit_settings(candidate, None)
    }

    pub fn clear_logs(&mut self) {
        self.state.logs.clear();
        self.bump_state();
    }

    pub fn activate(
        &mut self,
        app: PlatformAppId,
        generation: u64,
    ) -> Result<(), ApplicationError> {
        self.active_app = Some(app.clone());
        self.active_generation = generation;
        self.handled_generation = None;
        self.state.current_app = Some(app);
        self.state.matched_rule_id = None;
        self.state.current_input_source = None;
        self.state.target_input_source = None;
        self.bump_state();
        Ok(())
    }

    pub fn prepare_focus(
        &mut self,
        snapshot: FocusSnapshot,
    ) -> Result<PreparedFocus, ApplicationError> {
        let started = Instant::now();
        if !self.accepts(&snapshot) {
            let reason = self.reject_reason(&snapshot);
            // 重複フォーカスはノイズになりやすいので、デバッグ時のみ残す。
            if self.settings.debug_logging {
                self.push_log(
                    &snapshot,
                    "",
                    SwitchDecision::Ignored,
                    "",
                    "",
                    false,
                    false,
                    reason,
                    started,
                );
            }
            return Ok(PreparedFocus::Done(SwitchDecision::Ignored));
        }
        self.handled_generation = Some(snapshot.generation());
        self.state.current_app = Some(snapshot.app().clone());

        if !self.settings.enabled {
            self.push_log(
                &snapshot,
                "",
                SwitchDecision::Disabled,
                "",
                "",
                false,
                false,
                "auto switch is disabled".to_owned(),
                started,
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Disabled)));
        }
        if self
            .settings
            .snoozed_until_unix_ms
            .is_some_and(|deadline| deadline > self.clock.now_unix_ms())
        {
            let remaining = self
                .settings
                .snoozed_until_unix_ms
                .map(|deadline| deadline.saturating_sub(self.clock.now_unix_ms()))
                .unwrap_or(0);
            self.push_log(
                &snapshot,
                "",
                SwitchDecision::Snoozed,
                "",
                "",
                false,
                false,
                format!("snoozed for remaining {remaining}ms"),
                started,
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Snoozed)));
        }
        if self.settings.snoozed_until_unix_ms.is_some() {
            self.settings.snoozed_until_unix_ms = None;
            self.state.snoozed_until_unix_ms = None;
        }

        let Some(rule) = self.rules.first_match(&snapshot).cloned() else {
            self.state.matched_rule_id = None;
            self.state.target_input_source = None;
            self.push_log(
                &snapshot,
                "",
                SwitchDecision::NoRule,
                "",
                "",
                false,
                false,
                format!(
                    "no matching rule among {} enabled rules",
                    self.rules
                        .rules()
                        .iter()
                        .filter(|rule| rule.enabled())
                        .count()
                ),
                started,
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::NoRule)));
        };
        let Some(target) = self.resolve_target(rule.target()).cloned() else {
            let reason = match rule.target() {
                RuleTarget::PreferredLanguage(tag) => {
                    format!("preferred language '{}' is unset", tag.as_str())
                }
                RuleTarget::Exact(_) => "exact target unresolved".to_owned(),
            };
            self.state.matched_rule_id = Some(rule.id().to_owned());
            self.state.target_input_source = None;
            self.push_log(
                &snapshot,
                rule.id(),
                SwitchDecision::Failed,
                "",
                "",
                false,
                false,
                reason,
                started,
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Failed)));
        };
        Ok(PreparedFocus::NeedsSwitch {
            snapshot,
            rule: Box::new(rule),
            target,
        })
    }

    pub fn complete_focus(
        &mut self,
        snapshot: FocusSnapshot,
        rule: FocusRule,
        target: InputSourceId,
        current: InputSourceId,
        select_result: Result<(), GatewayError>,
    ) -> Result<SwitchDecision, ApplicationError> {
        let started = Instant::now();
        self.state.matched_rule_id = Some(rule.id().to_owned());
        self.state.current_input_source = Some(current.clone());
        self.state.target_input_source = Some(target.clone());
        if current == target {
            self.push_log(
                &snapshot,
                rule.id(),
                SwitchDecision::AlreadySelected,
                current.as_str(),
                target.as_str(),
                true,
                true,
                "current input source already matches target; reselected to refresh field context"
                    .to_owned(),
                started,
            );
            return Ok(self.finish(SwitchDecision::AlreadySelected));
        }

        match select_result {
            Ok(()) => {
                self.push_log(
                    &snapshot,
                    rule.id(),
                    SwitchDecision::Switched,
                    current.as_str(),
                    target.as_str(),
                    true,
                    true,
                    String::new(),
                    started,
                );
                self.state.current_input_source = Some(target);
                Ok(self.finish(SwitchDecision::Switched))
            }
            Err(error) => {
                let message = error.to_string();
                self.push_log(
                    &snapshot,
                    rule.id(),
                    SwitchDecision::Failed,
                    current.as_str(),
                    target.as_str(),
                    false,
                    true,
                    message.clone(),
                    started,
                );
                let _ = self.finish(SwitchDecision::Failed);
                Err(ApplicationError::Gateway(error))
            }
        }
    }

    /// gateway 呼び出し自体が失敗したとき（current/list 失敗など）。
    pub fn record_switch_pipeline_error(
        &mut self,
        snapshot: &FocusSnapshot,
        rule_id: &str,
        target: &InputSourceId,
        error: &ApplicationError,
    ) {
        self.push_log(
            snapshot,
            rule_id,
            SwitchDecision::Failed,
            "",
            target.as_str(),
            false,
            true,
            error.to_string(),
            Instant::now(),
        );
        let _ = self.finish(SwitchDecision::Failed);
    }

    pub fn handle_focus(
        &mut self,
        snapshot: FocusSnapshot,
    ) -> Result<SwitchDecision, ApplicationError> {
        let result = match self.prepare_focus(snapshot)? {
            PreparedFocus::Done(decision) => Ok(decision),
            PreparedFocus::NeedsSwitch {
                snapshot,
                rule,
                target,
            } => {
                let switch_result = {
                    let mut gateway = self.gateway.lock().map_err(|_| {
                        ApplicationError::InvalidSettings("gateway lock poisoned".to_owned())
                    })?;
                    switch_input_source(&mut *gateway, &target)
                };
                match switch_result {
                    Ok((current, select_result)) => {
                        self.complete_focus(snapshot, *rule, target, current, select_result)
                    }
                    Err(error) => {
                        self.record_switch_pipeline_error(&snapshot, rule.id(), &target, &error);
                        Err(error)
                    }
                }
            }
        };
        self.flush_publish();
        result
    }

    pub fn replace_rules(&mut self, rules: Vec<FocusRule>) -> Result<(), ApplicationError> {
        let validated = RuleSet::new(rules.clone())
            .map_err(|error| ApplicationError::InvalidSettings(error.to_string()))?;
        let mut candidate = self.settings.clone();
        candidate.rules = rules;
        self.commit_settings(candidate, Some(validated))
    }

    fn commit_settings(
        &mut self,
        candidate: SwitcherSettings,
        validated_rules: Option<RuleSet>,
    ) -> Result<(), ApplicationError> {
        candidate
            .validate()
            .map_err(|error| ApplicationError::InvalidSettings(error.to_string()))?;
        self.repository.save(&candidate)?;
        self.settings = candidate;
        if let Some(rules) = validated_rules {
            self.rules = rules;
            self.state.rules_version = self.state.rules_version.saturating_add(1);
        }
        self.bump_state();
        Ok(())
    }

    fn accepts(&self, snapshot: &FocusSnapshot) -> bool {
        let Some(active_app) = &self.active_app else {
            return false;
        };
        active_app.platform() == snapshot.app().platform()
            && active_app.app_id() == snapshot.app().app_id()
            && active_app.pid() == snapshot.app().pid()
            && snapshot.generation() >= self.active_generation
            && self.handled_generation != Some(snapshot.generation())
    }

    fn reject_reason(&self, snapshot: &FocusSnapshot) -> String {
        let Some(active_app) = &self.active_app else {
            return "no active app (waiting for activation)".to_owned();
        };
        if active_app.platform() != snapshot.app().platform()
            || active_app.app_id() != snapshot.app().app_id()
            || active_app.pid() != snapshot.app().pid()
        {
            return format!(
                "active app mismatch: active={} pid={} snapshot={} pid={}",
                active_app.app_id(),
                active_app.pid(),
                snapshot.app().app_id(),
                snapshot.app().pid()
            );
        }
        if snapshot.generation() < self.active_generation {
            return format!(
                "stale generation: snapshot={} active={}",
                snapshot.generation(),
                self.active_generation
            );
        }
        if self.handled_generation == Some(snapshot.generation()) {
            return format!("generation {} already handled", snapshot.generation());
        }
        "focus rejected".to_owned()
    }

    fn resolve_target<'a>(&'a self, target: &'a RuleTarget) -> Option<&'a InputSourceId> {
        match target {
            RuleTarget::PreferredLanguage(tag) => self.settings.preferred(tag),
            RuleTarget::Exact(id) => Some(id),
        }
    }

    fn focus_detail(snapshot: &FocusSnapshot) -> String {
        let elements = snapshot.elements();
        let Some(focused) = elements.first() else {
            return format!("elements=0 generation={}", snapshot.generation());
        };
        format!(
            "elements={} generation={} focused[role={} title={} id={} dom={} class={} desc={}]",
            elements.len(),
            snapshot.generation(),
            focused.value(AxAttribute::Role),
            focused.value(AxAttribute::Title),
            focused.value(AxAttribute::Identifier),
            focused.value(AxAttribute::DomIdentifier),
            focused.value(AxAttribute::DomClassList),
            focused.value(AxAttribute::Description),
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn push_log(
        &mut self,
        snapshot: &FocusSnapshot,
        rule_id: &str,
        decision: SwitchDecision,
        from: &str,
        to: &str,
        success: bool,
        attempted: bool,
        reason: String,
        started: Instant,
    ) {
        self.state.logs.insert(
            0,
            SwitchLogEntry {
                timestamp_unix_ms: self.clock.now_unix_ms(),
                app_id: snapshot.app().app_id().to_owned(),
                pid: snapshot.app().pid(),
                generation: snapshot.generation(),
                rule_id: rule_id.to_owned(),
                decision,
                from: from.to_owned(),
                to: to.to_owned(),
                success,
                attempted,
                reason,
                detail: Self::focus_detail(snapshot),
                processing_micros: u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX),
            },
        );
        self.state.logs.truncate(MAX_LOGS);
    }

    fn finish(&mut self, decision: SwitchDecision) -> SwitchDecision {
        self.state.last_decision = Some(decision);
        self.bump_state();
        decision
    }

    fn bump_state(&mut self) {
        self.state.version = self.state.version.saturating_add(1);
        self.state.enabled = self.settings.enabled;
        self.state.snoozed_until_unix_ms = self.settings.snoozed_until_unix_ms;
        self.publish_pending = true;
    }

    /// Take a publish snapshot if state changed. Call after releasing the
    /// switcher mutex, then invoke the event publisher.
    pub fn drain_publish_snapshot(&mut self) -> Option<(SwitcherState, String)> {
        if !self.publish_pending {
            return None;
        }
        self.publish_pending = false;
        Some((self.state.clone(), self.settings.ui_locale.clone()))
    }

    /// For in-process callers (unit tests / `handle_focus`) that already own
    /// `&mut self` without a shared mutex — flush pending publish immediately.
    pub fn flush_publish(&mut self) {
        if let Some((state, locale)) = self.drain_publish_snapshot() {
            self.publisher.publish(&state, &locale);
        }
    }
}

pub(crate) fn switch_input_source<G: InputSourceGateway>(
    gateway: &mut G,
    target: &InputSourceId,
) -> Result<(InputSourceId, Result<(), GatewayError>), ApplicationError> {
    let current = gateway.current()?;
    let select_result = match gateway.select(target) {
        Ok(()) => Ok(()),
        Err(_) => {
            gateway.list(true)?;
            gateway.select(target)
        }
    };
    Ok((current, select_result))
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    };

    use super::{InputSourceSwitcher, SwitchDecision};
    use crate::{
        application::{
            ports::{
                clock::Clock,
                input_source::{AvailableInputSource, GatewayError, InputSourceGateway},
                settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
                switcher_events::SwitcherEventPublisher,
            },
            settings::SwitcherSettings,
            switcher_state::SwitcherState,
        },
        domain::input_source::{
            FocusRule, FocusSnapshot, InputSourceId, LanguageTag, Platform, PlatformAppId,
            RuleTarget,
        },
    };

    const APP_ID: &str = "com.example.Editor";

    #[derive(Default)]
    struct GatewayData {
        current: String,
        selected: Vec<String>,
        select_attempts: usize,
        refreshes: usize,
        fail_first_select: bool,
    }

    struct FakeGateway(Arc<Mutex<GatewayData>>);

    impl InputSourceGateway for FakeGateway {
        fn list(&mut self, refresh: bool) -> Result<Vec<AvailableInputSource>, GatewayError> {
            if refresh {
                self.0.lock().expect("gateway lock").refreshes += 1;
            }
            Ok(Vec::new())
        }

        fn current(&self) -> Result<InputSourceId, GatewayError> {
            InputSourceId::parse(&self.0.lock().expect("gateway lock").current)
                .map_err(|error| GatewayError::Operation(error.to_string()))
        }

        fn select(&mut self, id: &InputSourceId) -> Result<(), GatewayError> {
            let mut data = self.0.lock().expect("gateway lock");
            data.select_attempts += 1;
            if data.fail_first_select && data.select_attempts == 1 {
                return Err(GatewayError::Operation("stale cache".to_owned()));
            }
            data.current = id.as_str().to_owned();
            data.selected.push(id.as_str().to_owned());
            Ok(())
        }
    }

    struct FakeRepository {
        fail_save: Arc<AtomicBool>,
    }

    impl SettingsRepository for FakeRepository {
        fn load(&self) -> Result<LoadOutcome, SettingsError> {
            Ok(LoadOutcome::Defaults {
                settings: SwitcherSettings::defaults(),
            })
        }

        fn save(&self, _settings: &SwitcherSettings) -> Result<(), SettingsError> {
            if self.fail_save.load(Ordering::Relaxed) {
                Err(SettingsError::Io("disk full".to_owned()))
            } else {
                Ok(())
            }
        }
    }

    struct FakeClock(Arc<AtomicU64>);

    impl Clock for FakeClock {
        fn now_unix_ms(&self) -> u64 {
            self.0.load(Ordering::Relaxed)
        }
    }

    struct FakePublisher(Arc<Mutex<Vec<u64>>>);

    impl SwitcherEventPublisher for FakePublisher {
        fn publish(&self, state: &SwitcherState, _locale: &str) {
            self.0.lock().expect("events lock").push(state.version);
        }
    }

    struct Fixture {
        switcher: InputSourceSwitcher<FakeGateway, FakeRepository, FakeClock, FakePublisher>,
        gateway: Arc<Mutex<GatewayData>>,
        fail_save: Arc<AtomicBool>,
        now: Arc<AtomicU64>,
        versions: Arc<Mutex<Vec<u64>>>,
    }

    impl Fixture {
        fn new(mut settings: SwitcherSettings) -> Self {
            settings.rules = vec![FocusRule::new(
                "editor",
                "Editor",
                true,
                Platform::MacOs,
                APP_ID,
                0,
                Vec::new(),
                RuleTarget::Exact(InputSourceId::parse("en").expect("valid id")),
            )
            .expect("valid rule")];
            let gateway = Arc::new(Mutex::new(GatewayData {
                current: "jp".to_owned(),
                ..GatewayData::default()
            }));
            let fail_save = Arc::new(AtomicBool::new(false));
            let now = Arc::new(AtomicU64::new(100));
            let versions = Arc::new(Mutex::new(Vec::new()));
            let switcher = InputSourceSwitcher::new(
                settings,
                Arc::new(Mutex::new(FakeGateway(Arc::clone(&gateway)))),
                FakeRepository {
                    fail_save: Arc::clone(&fail_save),
                },
                FakeClock(Arc::clone(&now)),
                FakePublisher(Arc::clone(&versions)),
            )
            .expect("valid switcher");
            Self {
                switcher,
                gateway,
                fail_save,
                now,
                versions,
            }
        }

        fn active() -> Self {
            let mut fixture = Self::new(SwitcherSettings::defaults());
            fixture.switcher.activate(app(42), 1).expect("activate app");
            fixture.switcher.flush_publish();
            fixture
        }
    }

    fn app(pid: i32) -> PlatformAppId {
        PlatformAppId::new(Platform::MacOs, APP_ID, pid).expect("valid app")
    }

    fn snapshot(pid: i32, generation: u64) -> FocusSnapshot {
        FocusSnapshot::new(app(pid), generation, Vec::new())
    }

    #[test]
    fn stale_generation_is_ignored() {
        let mut fixture = Fixture::new(SwitcherSettings::defaults());
        fixture.switcher.activate(app(42), 8).expect("activate");

        let decision = fixture
            .switcher
            .handle_focus(snapshot(42, 7))
            .expect("handle stale focus");

        assert_eq!(decision, SwitchDecision::Ignored);
        assert!(fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .selected
            .is_empty());
    }

    #[test]
    fn manual_change_is_respected_until_focus_changes() {
        let mut fixture = Fixture::active();

        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("first focus");
        fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("duplicate focus");
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en"]
        );

        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("new focus");
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en", "en"]
        );
    }

    #[test]
    fn failed_settings_save_leaves_active_rules_unchanged() {
        let mut fixture = Fixture::active();
        let before = fixture.switcher.state().rules_version;
        fixture.fail_save.store(true, Ordering::Relaxed);
        let replacement = vec![FocusRule::new(
            "replacement",
            "Replacement",
            true,
            Platform::MacOs,
            APP_ID,
            0,
            Vec::new(),
            RuleTarget::PreferredLanguage(LanguageTag::ja()),
        )
        .expect("valid rule")];

        assert!(fixture.switcher.replace_rules(replacement).is_err());
        assert_eq!(fixture.switcher.state().rules_version, before);
        assert_eq!(fixture.switcher.rules()[0].id(), "editor");
    }

    #[test]
    fn disabled_switcher_does_not_select() {
        let mut settings = SwitcherSettings::defaults();
        settings.enabled = false;
        let mut fixture = Fixture::new(settings);
        fixture.switcher.activate(app(42), 1).expect("activate");

        let decision = fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("handle");

        assert_eq!(decision, SwitchDecision::Disabled);
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            0
        );
    }

    #[test]
    fn active_snooze_blocks_and_expired_snooze_allows_switching() {
        let mut settings = SwitcherSettings::defaults();
        settings.snoozed_until_unix_ms = Some(200);
        let mut fixture = Fixture::new(settings);
        fixture.switcher.activate(app(42), 1).expect("activate");

        assert_eq!(
            fixture
                .switcher
                .handle_focus(snapshot(42, 1))
                .expect("snoozed"),
            SwitchDecision::Snoozed
        );
        fixture.now.store(201, Ordering::Relaxed);
        assert_eq!(
            fixture
                .switcher
                .handle_focus(snapshot(42, 2))
                .expect("expired"),
            SwitchDecision::Switched
        );
    }

    #[test]
    fn indefinite_snooze_blocks_until_resume() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .snooze_indefinitely()
            .expect("snooze indefinitely");
        // Still well below INDEFINITE_SNOOZE_UNTIL_UNIX_MS
        fixture.now.store(2_000_000_000_000, Ordering::Relaxed);

        assert_eq!(
            fixture
                .switcher
                .handle_focus(snapshot(42, 1))
                .expect("still snoozed"),
            SwitchDecision::Snoozed
        );

        fixture.switcher.set_snoozed_until(None).expect("resume");
        assert_eq!(
            fixture
                .switcher
                .handle_focus(snapshot(42, 2))
                .expect("resumed"),
            SwitchDecision::Switched
        );
    }

    #[test]
    fn same_source_still_reselects_to_refresh_context() {
        let mut fixture = Fixture::active();
        fixture.gateway.lock().expect("gateway lock").current = "en".to_owned();

        let decision = fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("handle");

        assert_eq!(decision, SwitchDecision::AlreadySelected);
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            1
        );
        let log = &fixture.switcher.state().logs[0];
        assert!(log.attempted);
        assert!(log.reason.contains("reselected"));
    }

    #[test]
    fn unmatched_application_is_a_no_op() {
        let mut fixture = Fixture::active();
        let other =
            PlatformAppId::new(Platform::MacOs, "com.example.Other", 7).expect("valid other app");
        fixture
            .switcher
            .activate(other.clone(), 2)
            .expect("activate");

        let decision = fixture
            .switcher
            .handle_focus(FocusSnapshot::new(other, 2, Vec::new()))
            .expect("handle");

        assert_eq!(decision, SwitchDecision::NoRule);
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            0
        );
        let log = &fixture.switcher.state().logs[0];
        assert_eq!(log.decision, SwitchDecision::NoRule);
        assert!(!log.attempted);
        assert!(log.reason.contains("no matching rule"));
        assert!(log.detail.contains("elements=0"));
    }

    #[test]
    fn switch_failure_refreshes_once_and_retries_once() {
        let mut fixture = Fixture::active();
        fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .fail_first_select = true;

        let decision = fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("retry succeeds");
        let data = fixture.gateway.lock().expect("gateway lock");

        assert_eq!(decision, SwitchDecision::Switched);
        assert_eq!(data.select_attempts, 2);
        assert_eq!(data.refreshes, 1);
    }

    #[test]
    fn switch_log_is_capped_at_two_hundred_entries() {
        let mut fixture = Fixture::active();
        for generation in 1..=205 {
            fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
            fixture
                .switcher
                .handle_focus(snapshot(42, generation))
                .expect("switch");
        }

        assert_eq!(fixture.switcher.state().logs.len(), 200);
    }

    #[test]
    fn published_state_versions_are_strictly_increasing() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("first");
        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("second");
        let versions = fixture.versions.lock().expect("versions lock");

        assert!(versions.windows(2).all(|pair| pair[0] < pair[1]));
    }
}
