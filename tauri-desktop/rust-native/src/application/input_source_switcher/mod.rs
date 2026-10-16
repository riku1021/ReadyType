use std::{
    fmt,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::domain::input_source::{
    is_ime_latin_mode, AxAttribute, FocusRule, FocusSnapshot, InputSourceId, LanguageTag,
    PlatformAppId, RuleSet, RuleTarget,
};

use super::{
    current_source_cache::CurrentInputSourceCache,
    ports::{
        clock::Clock,
        diagnostic_log::SharedDiagnosticLogSink,
        input_source::{GatewayError, InputSourceGateway},
        settings_repository::{SettingsError, SettingsRepository},
        switcher_events::SwitcherEventPublisher,
    },
    settings::SwitcherSettings,
    switch_ablation, switch_proto,
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
    /// Last UI-relevant publish fingerprint (PROTO_PUBLISH_FINGERPRINT).
    last_publish_fingerprint: Option<PublishFingerprint>,
    active_app: Option<PlatformAppId>,
    active_generation: u64,
    handled_generation: Option<u64>,
    /// Manual override latch (PROTO_MANUAL_OVERRIDE).
    current_focus_key: Option<String>,
    manual_override_key: Option<String>,
    last_auto_target: Option<InputSourceId>,
    suppress_tis_as_manual: bool,
    /// UI ログと並行してファイル等へ書く（未配線なら None）。
    diagnostic_log: Option<SharedDiagnosticLogSink>,
    /// Shared with TIS notify / switch_input_source (PROTO_CURRENT_CACHE).
    current_cache: Arc<Mutex<CurrentInputSourceCache>>,
}

/// UI / tray 向け publish の意味指紋（version / logs は除外）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishFingerprint {
    enabled: bool,
    snoozed_until_unix_ms: Option<u64>,
    current_app: Option<String>,
    matched_rule_id: Option<String>,
    current_input_source: Option<String>,
    target_input_source: Option<String>,
    last_decision: Option<SwitchDecision>,
}

impl PublishFingerprint {
    fn from_state(state: &SwitcherState) -> Self {
        Self {
            enabled: state.enabled,
            snoozed_until_unix_ms: state.snoozed_until_unix_ms,
            current_app: state
                .current_app
                .as_ref()
                .map(|app| format!("{}:{}", app.app_id(), app.pid())),
            matched_rule_id: state.matched_rule_id.clone(),
            current_input_source: state
                .current_input_source
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            target_input_source: state
                .target_input_source
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            last_decision: state.last_decision,
        }
    }
}

/// `gateway.select` 成功後の検証メモ（ログ reason 用）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectOutcome {
    pub note: String,
}

/// `switch_input_source` の区間計測。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SwitchSourceTiming {
    pub current_micros: u64,
    pub select_micros: u64,
    /// `select` を呼んだか（ablation でスキップしたら false）。micros が 0 でも区別する。
    pub select_attempted: bool,
}

/// ログ用の区間内訳。
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LogTiming {
    pub prepare_micros: u64,
    pub current_micros: u64,
    pub select_micros: u64,
}

/// `complete_focus` の引数まとめ（Clippy too_many_arguments 回避）。
pub struct CompleteFocusInput {
    pub snapshot: FocusSnapshot,
    pub rule: FocusRule,
    pub target: InputSourceId,
    pub current: InputSourceId,
    pub select_result: Result<SelectOutcome, GatewayError>,
    pub prepare_micros: u64,
    pub source_timing: SwitchSourceTiming,
}

/// `prepare_focus` の結果。gateway 呼び出しはロック外で行う。
pub enum PreparedFocus {
    Done(SwitchDecision),
    NeedsSwitch {
        snapshot: FocusSnapshot,
        rule: Box<FocusRule>,
        target: InputSourceId,
        prepare_micros: u64,
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
        let list_locale = SwitcherSettings::resolve_ui_locale(&settings.ui_locale);
        if let Ok(mut guard) = gateway.lock() {
            if let Ok(sources) = guard.list(false, &list_locale) {
                let seeded_or_reconciled = if settings.language_preferences.is_empty() {
                    let current = guard.current().ok();
                    super::settings::seed_language_preferences_from_sources(
                        &sources,
                        current.as_ref(),
                    )
                } else {
                    super::settings::reconcile_language_preferences(
                        &settings.language_preferences,
                        &sources,
                    )
                };
                let next_preferences = super::settings::ensure_preferences_for_rule_languages(
                    &seeded_or_reconciled,
                    &sources,
                    &settings.rules,
                );
                let preferences_changed = !next_preferences.is_empty()
                    && next_preferences != settings.language_preferences;
                if preferences_changed {
                    settings.language_preferences = next_preferences;
                }
                let names_changed = super::settings::merge_input_source_display_names(
                    &mut settings.input_source_display_names,
                    &sources,
                );
                if preferences_changed || names_changed {
                    match repository.save(&settings) {
                        Ok(()) => {}
                        // Corrupt recovery: keep in-memory seed, do not clobber quarantined data.
                        Err(SettingsError::WritesSuspended) => {}
                        Err(error) => return Err(ApplicationError::from(error)),
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
            last_publish_fingerprint: None,
            active_app: None,
            active_generation: 0,
            handled_generation: None,
            current_focus_key: None,
            manual_override_key: None,
            last_auto_target: None,
            suppress_tis_as_manual: false,
            diagnostic_log: None,
            current_cache: Arc::new(Mutex::new(CurrentInputSourceCache::new())),
        })
    }

    pub fn set_diagnostic_log_sink(&mut self, sink: Option<SharedDiagnosticLogSink>) {
        self.diagnostic_log = sink;
    }

    pub fn gateway_handle(&self) -> Arc<Mutex<G>> {
        Arc::clone(&self.gateway)
    }

    pub fn current_cache_handle(&self) -> Arc<Mutex<CurrentInputSourceCache>> {
        Arc::clone(&self.current_cache)
    }

    pub fn invalidate_current_cache(&self) {
        if let Ok(mut cache) = self.current_cache.lock() {
            cache.invalidate();
        }
    }

    pub fn warm_current_cache(&self) {
        if !switch_proto::current_cache() {
            return;
        }
        let Ok(gateway) = self.gateway.lock() else {
            return;
        };
        if let Ok(id) = gateway.current() {
            if let Ok(mut cache) = self.current_cache.lock() {
                cache.put(id);
            }
        }
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
        locale: &str,
    ) -> Result<Vec<super::ports::input_source::AvailableInputSource>, ApplicationError> {
        self.gateway
            .lock()
            .map_err(|_| ApplicationError::InvalidSettings("gateway lock poisoned".to_owned()))?
            .list(refresh, locale)
            .map_err(ApplicationError::from)
    }

    /// OS 一覧から表示名を取り込み、変化があれば保存する。
    /// `WritesSuspended` のときはメモリ上のみ更新する。
    pub fn record_input_source_display_names(
        &mut self,
        sources: &[super::ports::input_source::AvailableInputSource],
    ) -> Result<(), ApplicationError> {
        if !super::settings::merge_input_source_display_names(
            &mut self.settings.input_source_display_names,
            sources,
        ) {
            return Ok(());
        }
        match self.repository.save(&self.settings) {
            Ok(()) | Err(SettingsError::WritesSuspended) => Ok(()),
            Err(error) => Err(ApplicationError::from(error)),
        }
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

    /// `shortcut` は呼び出し側で正規化済みであること。
    pub fn set_quick_set_shortcut(&mut self, shortcut: &str) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.quick_set_shortcut = shortcut.to_owned();
        self.commit_settings(candidate, None)
    }

    pub fn complete_onboarding(&mut self) -> Result<(), ApplicationError> {
        let mut candidate = self.settings.clone();
        candidate.onboarding_completed = true;
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
        if switch_proto::manual_override() {
            self.current_focus_key = None;
            self.manual_override_key = None;
            self.last_auto_target = None;
            self.suppress_tis_as_manual = false;
        }
        self.state.current_app = Some(app);
        self.state.matched_rule_id = None;
        self.state.current_input_source = None;
        self.state.target_input_source = None;
        self.bump_state();
        Ok(())
    }

    /// External TIS change (PROTO_TIS_NOTIFY / PROTO_MANUAL_OVERRIDE).
    pub fn note_external_input_source_change(&mut self, current: &InputSourceId) {
        if switch_proto::manual_override()
            && !self.suppress_tis_as_manual
            && self.last_auto_target.as_ref() != Some(current)
        {
            if let Some(key) = self.current_focus_key.clone() {
                self.manual_override_key = Some(key);
            }
        }
    }

    pub fn set_suppress_tis_as_manual(&mut self, suppress: bool) {
        self.suppress_tis_as_manual = suppress;
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
                    LogTiming {
                        prepare_micros: micros_since(started),
                        ..LogTiming::default()
                    },
                );
            }
            return Ok(PreparedFocus::Done(SwitchDecision::Ignored));
        }
        self.handled_generation = Some(snapshot.generation());
        self.state.current_app = Some(snapshot.app().clone());

        if switch_proto::manual_override() {
            let focus_key = snapshot.focus_key();
            if self.current_focus_key.as_deref() != Some(focus_key.as_str()) {
                self.manual_override_key = None;
            }
            self.current_focus_key = Some(focus_key.clone());
            if self.manual_override_key.as_deref() == Some(focus_key.as_str()) {
                self.push_log(
                    &snapshot,
                    "",
                    SwitchDecision::Ignored,
                    "",
                    "",
                    true,
                    false,
                    "manual override latch: same focus context; select skipped".to_owned(),
                    started,
                    LogTiming {
                        prepare_micros: micros_since(started),
                        ..LogTiming::default()
                    },
                );
                return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Ignored)));
            }
        }

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
                LogTiming {
                    prepare_micros: micros_since(started),
                    ..LogTiming::default()
                },
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
                LogTiming {
                    prepare_micros: micros_since(started),
                    ..LogTiming::default()
                },
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
                LogTiming {
                    prepare_micros: micros_since(started),
                    ..LogTiming::default()
                },
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
                LogTiming {
                    prepare_micros: micros_since(started),
                    ..LogTiming::default()
                },
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Failed)));
        };
        Ok(PreparedFocus::NeedsSwitch {
            snapshot,
            rule: Box::new(rule),
            target,
            prepare_micros: micros_since(started),
        })
    }

    pub fn complete_focus(
        &mut self,
        input: CompleteFocusInput,
    ) -> Result<SwitchDecision, ApplicationError> {
        let CompleteFocusInput {
            snapshot,
            rule,
            target,
            current,
            select_result,
            prepare_micros,
            source_timing,
        } = input;
        let started = Instant::now();
        let timing = LogTiming {
            prepare_micros,
            current_micros: source_timing.current_micros,
            select_micros: source_timing.select_micros,
        };
        self.state.matched_rule_id = Some(rule.id().to_owned());
        self.state.current_input_source = Some(current.clone());
        self.state.target_input_source = Some(target.clone());
        if current == target {
            let reason = if !source_timing.select_attempted {
                Self::with_source_tags(
                    "current input source already matches target; same-source reselect skipped (ablation)",
                    current.as_str(),
                    target.as_str(),
                )
            } else {
                Self::with_source_tags(
                    "current input source already matches target; reselected to refresh field context",
                    current.as_str(),
                    target.as_str(),
                )
            };
            self.push_log(
                &snapshot,
                rule.id(),
                SwitchDecision::AlreadySelected,
                current.as_str(),
                target.as_str(),
                true,
                source_timing.select_attempted,
                reason,
                started,
                timing,
            );
            if switch_proto::manual_override() {
                self.last_auto_target = Some(target);
            }
            return Ok(self.finish(SwitchDecision::AlreadySelected));
        }

        match select_result {
            Ok(outcome) => {
                let mut reason = Self::with_source_tags(
                    "selected target input source",
                    current.as_str(),
                    target.as_str(),
                );
                if !outcome.note.is_empty() {
                    reason.push_str("; ");
                    reason.push_str(&outcome.note);
                }
                self.push_log(
                    &snapshot,
                    rule.id(),
                    SwitchDecision::Switched,
                    current.as_str(),
                    target.as_str(),
                    true,
                    true,
                    reason,
                    started,
                    timing,
                );
                if switch_proto::manual_override() {
                    self.last_auto_target = Some(target.clone());
                }
                self.state.current_input_source = Some(target);
                Ok(self.finish(SwitchDecision::Switched))
            }
            Err(error) => {
                let message =
                    Self::with_source_tags(&error.to_string(), current.as_str(), target.as_str());
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
                    timing,
                );
                let _ = self.finish(SwitchDecision::Failed);
                Err(ApplicationError::Gateway(error))
            }
        }
    }

    /// follow-up reselect の結果（debug_logging 時のみ残す）。
    pub fn note_followup_reselect(
        &mut self,
        delay_ms: u64,
        status: i32,
        current: Option<&InputSourceId>,
    ) {
        if !self.settings.debug_logging {
            return;
        }
        let current = current
            .map(|id| {
                if is_ime_latin_mode(id.as_str()) {
                    format!("{} [imeLatin]", id.as_str())
                } else {
                    id.as_str().to_owned()
                }
            })
            .unwrap_or_else(|| "—".to_owned());
        self.push_control_log(
            format!("tis: followup reselect +{delay_ms}ms status={status} current={current}"),
            String::new(),
        );
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
            LogTiming::default(),
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
                prepare_micros,
            } => {
                if switch_proto::manual_override() {
                    self.suppress_tis_as_manual = true;
                }
                let switch_result = {
                    let mut gateway = self.gateway.lock().map_err(|_| {
                        ApplicationError::InvalidSettings("gateway lock poisoned".to_owned())
                    })?;
                    let locale = SwitcherSettings::resolve_ui_locale(&self.settings.ui_locale);
                    switch_input_source(&mut *gateway, &target, &locale, Some(&self.current_cache))
                };
                if switch_proto::manual_override() {
                    self.suppress_tis_as_manual = false;
                }
                match switch_result {
                    Ok((current, select_result, source_timing)) => {
                        self.complete_focus(CompleteFocusInput {
                            snapshot,
                            rule: *rule,
                            target,
                            current,
                            select_result,
                            prepare_micros,
                            source_timing,
                        })
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
        self.commit_rules(rules, None)
    }

    /// Replace rules and upsert a language preference in a single settings commit.
    pub fn replace_rules_with_language_preference(
        &mut self,
        rules: Vec<FocusRule>,
        tag: &LanguageTag,
        source_id: InputSourceId,
    ) -> Result<(), ApplicationError> {
        self.commit_rules(rules, Some((tag, source_id)))
    }

    fn commit_rules(
        &mut self,
        rules: Vec<FocusRule>,
        language_preference: Option<(&LanguageTag, InputSourceId)>,
    ) -> Result<(), ApplicationError> {
        let validated = RuleSet::new(rules.clone())
            .map_err(|error| ApplicationError::InvalidSettings(error.to_string()))?;
        let mut candidate = self.settings.clone();
        if let Some((tag, source_id)) = language_preference {
            if candidate.preferred(tag) != Some(&source_id) {
                candidate
                    .language_preferences
                    .insert(tag.clone(), Some(source_id));
            }
        }
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
        let mut focused_parts = Vec::new();
        for (label, attribute) in [
            ("role", AxAttribute::Role),
            ("id", AxAttribute::Identifier),
            ("dom", AxAttribute::DomIdentifier),
            ("class", AxAttribute::DomClassList),
            ("title", AxAttribute::Title),
            ("desc", AxAttribute::Description),
        ] {
            let value = focused.value(attribute);
            if !value.is_empty() {
                focused_parts.push(format!("{label}={value}"));
            }
        }
        format!(
            "elements={} generation={} focused[{}]",
            elements.len(),
            snapshot.generation(),
            focused_parts.join(" ")
        )
    }

    fn with_source_tags(base: &str, from: &str, to: &str) -> String {
        let mut tags = Vec::new();
        if !from.is_empty() && is_ime_latin_mode(from) {
            tags.push("from=imeLatin");
        }
        if !to.is_empty() && is_ime_latin_mode(to) {
            tags.push("to=imeLatin");
        }
        if tags.is_empty() {
            base.to_owned()
        } else {
            format!("{base} ({})", tags.join(" "))
        }
    }

    fn push_control_log(&mut self, reason: String, detail: String) {
        let (app_id, pid) = self
            .state
            .current_app
            .as_ref()
            .map(|app| (app.app_id().to_owned(), app.pid()))
            .unwrap_or_else(|| (String::new(), 0));
        self.state.logs.insert(
            0,
            SwitchLogEntry {
                timestamp_unix_ms: self.clock.now_unix_ms(),
                app_id,
                pid,
                generation: self.handled_generation.unwrap_or(0),
                rule_id: String::new(),
                decision: SwitchDecision::Ignored,
                from: String::new(),
                to: String::new(),
                success: true,
                attempted: false,
                reason,
                detail,
                processing_micros: 0,
                prepare_micros: 0,
                current_micros: 0,
                select_micros: 0,
            },
        );
        self.state.logs.truncate(MAX_LOGS);
        self.persist_latest_log();
    }

    fn persist_latest_log(&self) {
        let Some(sink) = &self.diagnostic_log else {
            return;
        };
        let Some(entry) = self.state.logs.first() else {
            return;
        };
        sink.append_switch_log(entry);
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
        timing: LogTiming,
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
                processing_micros: micros_since(started),
                prepare_micros: timing.prepare_micros,
                current_micros: timing.current_micros,
                select_micros: timing.select_micros,
            },
        );
        self.state.logs.truncate(MAX_LOGS);
        self.persist_latest_log();
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
        if switch_proto::publish_fingerprint() {
            let fingerprint = PublishFingerprint::from_state(&self.state);
            if self.last_publish_fingerprint.as_ref() == Some(&fingerprint) {
                return None;
            }
            self.last_publish_fingerprint = Some(fingerprint);
        }
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
    locale: &str,
    current_cache: Option<&Mutex<CurrentInputSourceCache>>,
) -> Result<
    (
        InputSourceId,
        Result<SelectOutcome, GatewayError>,
        SwitchSourceTiming,
    ),
    ApplicationError,
> {
    let current_started = Instant::now();
    let current = if switch_proto::current_cache() {
        if let Some(cache) = current_cache {
            if let Ok(guard) = cache.lock() {
                if let Some(cached) = guard.get() {
                    let id = cached.clone();
                    drop(guard);
                    let current_micros = micros_since(current_started);
                    return finish_switch_after_current(
                        gateway,
                        target,
                        locale,
                        current_cache,
                        id,
                        current_micros,
                    );
                }
            }
        }
        let id = gateway.current()?;
        if let Some(cache) = current_cache {
            if let Ok(mut guard) = cache.lock() {
                guard.put(id.clone());
            }
        }
        id
    } else {
        gateway.current()?
    };
    let current_micros = micros_since(current_started);
    finish_switch_after_current(
        gateway,
        target,
        locale,
        current_cache,
        current,
        current_micros,
    )
}

fn finish_switch_after_current<G: InputSourceGateway>(
    gateway: &mut G,
    target: &InputSourceId,
    locale: &str,
    current_cache: Option<&Mutex<CurrentInputSourceCache>>,
    current: InputSourceId,
    current_micros: u64,
) -> Result<
    (
        InputSourceId,
        Result<SelectOutcome, GatewayError>,
        SwitchSourceTiming,
    ),
    ApplicationError,
> {
    if current == *target && switch_ablation::no_same_source_reselect() {
        return Ok((
            current,
            Ok(SelectOutcome {
                note: "same-source reselect skipped (ablation)".to_owned(),
            }),
            SwitchSourceTiming {
                current_micros,
                select_micros: 0,
                select_attempted: false,
            },
        ));
    }

    let select_started = Instant::now();
    let select_result = match gateway.select(target) {
        Ok(()) => Ok(()),
        Err(first) => {
            if let Some(cache) = current_cache {
                if let Ok(mut guard) = cache.lock() {
                    guard.invalidate();
                }
            }
            if switch_ablation::no_retry() {
                Err(first)
            } else {
                gateway.list(true, locale)?;
                gateway.select(target)
            }
        }
    };
    let select_micros = micros_since(select_started);
    let timing = SwitchSourceTiming {
        current_micros,
        select_micros,
        select_attempted: true,
    };
    let outcome = select_result.map(|()| {
        if let Some(cache) = current_cache {
            if let Ok(mut guard) = cache.lock() {
                guard.put(target.clone());
            }
        }
        // select 成功でもメニューだけ先に変わることがあるので current を再読取して検証する。
        let mut notes = Vec::new();
        match gateway.current().ok() {
            Some(id) if id == *target => notes.push("postSelect=match".to_owned()),
            Some(id) => notes.push(format!("postSelect=mismatch current={}", id.as_str())),
            None => notes.push("postSelect=unreadable".to_owned()),
        }
        if let Some(override_id) = gateway.input_method_keyboard_layout_override() {
            notes.push(format!("layoutOverride={}", override_id.as_str()));
        }
        SelectOutcome {
            note: notes.join(" "),
        }
    });
    if outcome.is_err() {
        if let Some(cache) = current_cache {
            if let Ok(mut guard) = cache.lock() {
                guard.invalidate();
            }
        }
    }
    Ok((current, outcome, timing))
}

fn micros_since(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_micros()).unwrap_or(u64::MAX)
}

#[cfg(test)]
mod tests;
