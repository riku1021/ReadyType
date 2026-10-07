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
    current_cache: Arc<Mutex<CurrentInputSourceCache>>,
    repository: R,
    clock: C,
    publisher: E,
    state: SwitcherState,
    /// Set by `bump_state`; drained outside the switcher mutex before publish.
    publish_pending: bool,
    /// tray / FE 向けに意味のある差分が無い publish を省略するための指紋。
    last_publish_fingerprint: Option<PublishFingerprint>,
    active_app: Option<PlatformAppId>,
    active_generation: u64,
    handled_generation: Option<u64>,
    /// 直近に処理した正規化フォーカス文脈キー。
    current_focus_key: Option<String>,
    /// 手動 IME 切替を尊重するフォーカス文脈（xxkb 相当のラッチ）。
    manual_override_key: Option<String>,
    /// ReadyType が最後に select した（または一致確認した）ターゲット。
    last_auto_target: Option<InputSourceId>,
    /// 自前 select 起因の TIS 通知を手動扱いしない。
    suppress_tis_as_manual: bool,
    /// UI ログと並行してファイル等へ書く（未配線なら None）。
    diagnostic_log: Option<SharedDiagnosticLogSink>,
}

/// StateDto / tray に効くフィールド（version・logs は除外）。
#[derive(Debug, Clone, PartialEq, Eq)]
struct PublishFingerprint {
    enabled: bool,
    snoozed_until_unix_ms: Option<u64>,
    current_app: Option<PlatformAppId>,
    matched_rule_id: Option<String>,
    current_input_source: Option<InputSourceId>,
    target_input_source: Option<InputSourceId>,
    last_decision: Option<SwitchDecision>,
}

impl PublishFingerprint {
    fn from_state(state: &SwitcherState) -> Self {
        Self {
            enabled: state.enabled,
            snoozed_until_unix_ms: state.snoozed_until_unix_ms,
            current_app: state.current_app.clone(),
            matched_rule_id: state.matched_rule_id.clone(),
            current_input_source: state.current_input_source.clone(),
            target_input_source: state.target_input_source.clone(),
            last_decision: state.last_decision,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FocusPipelineTiming {
    pub pipeline_started: Instant,
    pub queue_micros: u64,
    pub prepare_micros: u64,
    pub handoff_micros: u64,
    pub current_micros: u64,
    pub current_cache_hit: bool,
    pub select_micros: u64,
    pub ax_micros: u64,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SwitchGatewayTiming {
    pub current_micros: u64,
    pub current_cache_hit: bool,
    pub select_micros: u64,
}

/// `gateway.select` 成功後の検証メモ（ログ reason 用）。
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SelectOutcome {
    pub note: String,
}

/// `prepare_focus` の結果。gateway 呼び出しはロック外で行う。
pub enum PreparedFocus {
    Done(SwitchDecision),
    NeedsSwitch {
        snapshot: FocusSnapshot,
        rule: Box<FocusRule>,
        target: InputSourceId,
        timing: FocusPipelineTiming,
        /// prepare で付いた接頭辞（例: override cleared）。complete_focus で reason に付ける。
        reason_prefix: String,
    },
}

fn micros_elapsed(since: Instant) -> u64 {
    u64::try_from(since.elapsed().as_micros()).unwrap_or(u64::MAX)
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
                let next_preferences = if settings.language_preferences.is_empty() {
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
                if !next_preferences.is_empty() && next_preferences != settings.language_preferences
                {
                    settings.language_preferences = next_preferences;
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
            current_cache: Arc::new(Mutex::new(CurrentInputSourceCache::default())),
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

    /// 自前 `select` の前後で呼ぶ。TIS 通知を manual override にしない。
    pub fn set_suppress_tis_as_manual(&mut self, suppress: bool) {
        self.suppress_tis_as_manual = suppress;
    }

    pub fn remember_auto_target(&mut self, target: &InputSourceId) {
        self.last_auto_target = Some(target.clone());
    }

    /// TIS 変更通知（キャッシュ invalidate 後）。外部変更なら現在のフォーカス文脈に override をラッチ。
    pub fn note_external_input_source_change(&mut self) {
        if self.suppress_tis_as_manual {
            // 自前 select のたびに飛ぶのでデバッグ時のみ。
            if self.settings.debug_logging {
                self.push_control_log(
                    "tis: ignored (suppress during select)".to_owned(),
                    self.tis_detail(None),
                );
            }
            return;
        }
        let current = self
            .gateway
            .lock()
            .ok()
            .and_then(|gateway| gateway.current().ok());
        if let Some(last) = &self.last_auto_target {
            if current.as_ref() == Some(last) {
                if self.settings.debug_logging {
                    self.push_control_log(
                        "tis: ignored (matches last_auto_target)".to_owned(),
                        self.tis_detail(current.as_ref()),
                    );
                }
                return;
            }
        }
        if let Some(key) = self.current_focus_key.clone() {
            self.manual_override_key = Some(key);
            self.push_control_log(
                "tis: external change latched as manual override".to_owned(),
                self.tis_detail(current.as_ref()),
            );
            return;
        }
        self.push_control_log(
            "tis: external change with no focus_key (override not latched)".to_owned(),
            self.tis_detail(current.as_ref()),
        );
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
        self.current_focus_key = None;
        self.manual_override_key = None;
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
        pipeline_started: Instant,
        ax_micros: u64,
    ) -> Result<PreparedFocus, ApplicationError> {
        let queue_micros = micros_elapsed(pipeline_started);
        let started = Instant::now();
        let judgment_timing = || FocusPipelineTiming {
            pipeline_started,
            queue_micros,
            prepare_micros: micros_elapsed(started),
            handoff_micros: 0,
            current_micros: 0,
            current_cache_hit: false,
            select_micros: 0,
            ax_micros,
        };
        if !self.accepts(&snapshot) {
            let reason = self.reject_reason(&snapshot);
            // 同一 generation の再配信は診断ノイズなので出さない。
            // それ以外の reject はデバッグ時のみ。
            if self.settings.debug_logging && !reason.starts_with("generation ") {
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
                    judgment_timing(),
                );
            }
            return Ok(PreparedFocus::Done(SwitchDecision::Ignored));
        }
        self.handled_generation = Some(snapshot.generation());
        self.state.current_app = Some(snapshot.app().clone());
        let focus_key = snapshot.focus_key().to_owned();
        let cleared_override = self.current_focus_key.as_deref() != Some(focus_key.as_str())
            && self.manual_override_key.is_some();
        if self.current_focus_key.as_deref() != Some(focus_key.as_str()) {
            self.manual_override_key = None;
        }
        self.current_focus_key = Some(focus_key.clone());
        let override_prefix = if cleared_override {
            "override cleared (focus_key changed); "
        } else {
            ""
        };

        if !self.settings.enabled {
            self.push_log(
                &snapshot,
                "",
                SwitchDecision::Disabled,
                "",
                "",
                false,
                false,
                format!("{override_prefix}auto switch is disabled"),
                started,
                judgment_timing(),
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
                format!("{override_prefix}snoozed for remaining {remaining}ms"),
                started,
                judgment_timing(),
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
                    "{override_prefix}no matching rule among {} enabled rules",
                    self.rules
                        .rules()
                        .iter()
                        .filter(|rule| rule.enabled())
                        .count()
                ),
                started,
                judgment_timing(),
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
                format!("{override_prefix}{reason}"),
                started,
                judgment_timing(),
            );
            return Ok(PreparedFocus::Done(self.finish(SwitchDecision::Failed)));
        };
        if self.manual_override_key.as_deref() == Some(focus_key.as_str()) {
            self.state.matched_rule_id = Some(rule.id().to_owned());
            self.state.target_input_source = Some(target.clone());
            self.push_log(
                &snapshot,
                rule.id(),
                SwitchDecision::AlreadySelected,
                "",
                target.as_str(),
                true,
                false,
                format!(
                    "{override_prefix}manual input source change respected for this focus; select skipped"
                ),
                started,
                judgment_timing(),
            );
            return Ok(PreparedFocus::Done(
                self.finish(SwitchDecision::AlreadySelected),
            ));
        }
        let prepare_micros = micros_elapsed(started);
        Ok(PreparedFocus::NeedsSwitch {
            snapshot,
            rule: Box::new(rule),
            target,
            timing: FocusPipelineTiming {
                pipeline_started,
                queue_micros,
                prepare_micros,
                handoff_micros: 0,
                current_micros: 0,
                current_cache_hit: false,
                select_micros: 0,
                ax_micros,
            },
            reason_prefix: override_prefix.to_owned(),
        })
    }

    pub fn complete_focus(
        &mut self,
        snapshot: FocusSnapshot,
        rule: FocusRule,
        target: InputSourceId,
        current: InputSourceId,
        select_result: Result<SelectOutcome, GatewayError>,
        timing: FocusPipelineTiming,
        reason_prefix: &str,
    ) -> Result<SwitchDecision, ApplicationError> {
        let started = Instant::now();
        self.state.matched_rule_id = Some(rule.id().to_owned());
        self.state.current_input_source = Some(current.clone());
        self.state.target_input_source = Some(target.clone());
        if current == target {
            self.remember_auto_target(&target);
            self.push_log(
                &snapshot,
                rule.id(),
                SwitchDecision::AlreadySelected,
                current.as_str(),
                target.as_str(),
                true,
                false,
                format!(
                    "{reason_prefix}{}",
                    Self::with_source_tags(
                        "current input source already matches target; select skipped",
                        current.as_str(),
                        target.as_str(),
                    )
                ),
                started,
                timing,
            );
            return Ok(self.finish(SwitchDecision::AlreadySelected));
        }

        match select_result {
            Ok(outcome) => {
                self.remember_auto_target(&target);
                let mut reason = format!(
                    "{reason_prefix}{}",
                    Self::with_source_tags(
                        "selected target input source",
                        current.as_str(),
                        target.as_str(),
                    )
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
                self.state.current_input_source = Some(target);
                Ok(self.finish(SwitchDecision::Switched))
            }
            Err(error) => {
                let message = format!(
                    "{reason_prefix}{}",
                    Self::with_source_tags(&error.to_string(), current.as_str(), target.as_str())
                );
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

    /// フォーカスキュー合流で中間イベントを捨てたとき。
    pub fn note_focus_coalesced(&mut self, dropped: u32, kept_generation: u64) {
        if dropped == 0 {
            return;
        }
        self.push_control_log(
            format!("focus: coalesced {dropped} queued event(s); kept gen={kept_generation}"),
            String::new(),
        );
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
        timing: FocusPipelineTiming,
        reason_prefix: &str,
    ) {
        let started = Instant::now();
        self.push_log(
            snapshot,
            rule_id,
            SwitchDecision::Failed,
            "",
            target.as_str(),
            false,
            true,
            format!("{reason_prefix}{error}"),
            started,
            timing,
        );
        let _ = self.finish(SwitchDecision::Failed);
    }

    pub fn handle_focus(
        &mut self,
        snapshot: FocusSnapshot,
    ) -> Result<SwitchDecision, ApplicationError> {
        let pipeline_started = Instant::now();
        let result = match self.prepare_focus(snapshot, pipeline_started, 0)? {
            PreparedFocus::Done(decision) => Ok(decision),
            PreparedFocus::NeedsSwitch {
                snapshot,
                rule,
                target,
                timing,
                reason_prefix,
            } => {
                let handoff_started = Instant::now();
                self.set_suppress_tis_as_manual(true);
                let switch_result = {
                    let mut gateway = self.gateway.lock().map_err(|_| {
                        ApplicationError::InvalidSettings("gateway lock poisoned".to_owned())
                    })?;
                    let mut cache = self.current_cache.lock().map_err(|_| {
                        ApplicationError::InvalidSettings("current cache lock poisoned".to_owned())
                    })?;
                    let handoff_micros = micros_elapsed(handoff_started);
                    let locale = SwitcherSettings::resolve_ui_locale(&self.settings.ui_locale);
                    let result = switch_input_source(&mut *gateway, &target, &locale, &mut cache);
                    (handoff_micros, result)
                };
                self.set_suppress_tis_as_manual(false);
                match switch_result {
                    (handoff_micros, Ok((current, select_outcome, gateway_timing))) => {
                        let timing = FocusPipelineTiming {
                            handoff_micros,
                            current_micros: gateway_timing.current_micros,
                            current_cache_hit: gateway_timing.current_cache_hit,
                            select_micros: gateway_timing.select_micros,
                            ..timing
                        };
                        self.complete_focus(
                            snapshot,
                            *rule,
                            target,
                            current,
                            select_outcome,
                            timing,
                            &reason_prefix,
                        )
                    }
                    (handoff_micros, Err(error)) => {
                        let timing = FocusPipelineTiming {
                            handoff_micros,
                            ..timing
                        };
                        self.record_switch_pipeline_error(
                            &snapshot,
                            rule.id(),
                            &target,
                            &error,
                            timing,
                            &reason_prefix,
                        );
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

    fn focus_detail(&self, snapshot: &FocusSnapshot) -> String {
        let override_state = if self.manual_override_key.as_deref() == Some(snapshot.focus_key()) {
            "active"
        } else if self.manual_override_key.is_some() {
            "otherKey"
        } else {
            "none"
        };
        let elements = snapshot.elements();
        let Some(focused) = elements.first() else {
            return format!("override={override_state} elements=0");
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
            "override={override_state} elements={} focused[{}]",
            elements.len(),
            focused_parts.join(" ")
        )
    }

    fn tis_detail(&self, current: Option<&InputSourceId>) -> String {
        let mut parts = Vec::new();
        if let Some(current) = current {
            parts.push(format!(
                "current={}{}",
                current.as_str(),
                if is_ime_latin_mode(current.as_str()) {
                    " [imeLatin]"
                } else {
                    ""
                }
            ));
        }
        if let Some(last) = &self.last_auto_target {
            parts.push(format!("lastAuto={}", last.as_str()));
        }
        parts.push(format!(
            "override={}",
            if self.manual_override_key.is_some() {
                "active"
            } else {
                "none"
            }
        ));
        parts.push(format!("suppress={}", self.suppress_tis_as_manual));
        parts.join(" ")
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
                focus_key: self.current_focus_key.clone().unwrap_or_default(),
                rule_id: String::new(),
                decision: SwitchDecision::Ignored,
                from: String::new(),
                to: String::new(),
                success: true,
                attempted: false,
                reason,
                detail,
                processing_micros: 0,
                pipeline_micros: 0,
                queue_micros: 0,
                prepare_micros: 0,
                handoff_micros: 0,
                current_micros: 0,
                current_cache_hit: false,
                select_micros: 0,
                finalize_micros: 0,
                publish_micros: 0,
                ax_micros: 0,
            },
        );
        self.state.logs.truncate(MAX_LOGS);
        self.persist_latest_log();
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
        timing: FocusPipelineTiming,
    ) {
        self.state.logs.insert(
            0,
            SwitchLogEntry {
                timestamp_unix_ms: self.clock.now_unix_ms(),
                app_id: snapshot.app().app_id().to_owned(),
                pid: snapshot.app().pid(),
                generation: snapshot.generation(),
                focus_key: snapshot.focus_key().to_owned(),
                rule_id: rule_id.to_owned(),
                decision,
                from: from.to_owned(),
                to: to.to_owned(),
                success,
                attempted,
                reason,
                detail: self.focus_detail(snapshot),
                processing_micros: micros_elapsed(started),
                pipeline_micros: micros_elapsed(timing.pipeline_started),
                queue_micros: timing.queue_micros,
                prepare_micros: timing.prepare_micros,
                handoff_micros: timing.handoff_micros,
                current_micros: timing.current_micros,
                current_cache_hit: timing.current_cache_hit,
                select_micros: timing.select_micros,
                finalize_micros: micros_elapsed(started),
                publish_micros: 0,
                ax_micros: timing.ax_micros,
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

    /// 直近ログに publish 所要を書き戻す（push_log 後に計測するため）。
    pub fn set_latest_log_publish_micros(&mut self, publish_micros: u64) {
        if let Some(log) = self.state.logs.first_mut() {
            log.publish_micros = publish_micros;
        }
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

    /// Take a publish snapshot if tray/FE-visible state changed.
    /// version / logs のみの更新は省略する（diagnostics は list_switch_logs で取得）。
    pub fn drain_publish_snapshot(&mut self) -> Option<(SwitcherState, String)> {
        if !self.publish_pending {
            return None;
        }
        self.publish_pending = false;
        let fingerprint = PublishFingerprint::from_state(&self.state);
        if self.last_publish_fingerprint.as_ref() == Some(&fingerprint) {
            return None;
        }
        self.last_publish_fingerprint = Some(fingerprint);
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
    cache: &mut CurrentInputSourceCache,
) -> Result<(InputSourceId, Result<SelectOutcome, GatewayError>, SwitchGatewayTiming), ApplicationError>
{
    let current_started = Instant::now();
    let now = Instant::now();
    let (current, current_cache_hit) = if let Some(cached) = cache.get(now) {
        (cached, true)
    } else {
        let fetched = gateway.current()?;
        cache.put(now, fetched.clone());
        (fetched, false)
    };
    let current_micros = micros_elapsed(current_started);
    if &current == target {
        // 既に目標ソースなら select しない（フィールド文脈の再選択コストを避ける）。
        return Ok((
            current,
            Ok(SelectOutcome::default()),
            SwitchGatewayTiming {
                current_micros,
                current_cache_hit,
                select_micros: 0,
            },
        ));
    }
    let select_started = Instant::now();
    let select_result = match gateway.select(target) {
        Ok(()) => Ok(()),
        Err(_) => {
            gateway.list(true, locale)?;
            gateway.select(target)
        }
    };
    let outcome = match select_result {
        Ok(()) => {
            // select 成功でもメニューだけ先に変わることがあるので current を再読取して検証する。
            cache.invalidate();
            let post = gateway.current().ok();
            if let Some(ref id) = post {
                cache.put(Instant::now(), id.clone());
            } else {
                cache.put(Instant::now(), target.clone());
            }
            let mut notes = Vec::new();
            match &post {
                Some(id) if id == target => notes.push("postSelect=match".to_owned()),
                Some(id) => notes.push(format!("postSelect=mismatch current={}", id.as_str())),
                None => notes.push("postSelect=unreadable".to_owned()),
            }
            if let Some(override_id) = gateway.input_method_keyboard_layout_override() {
                notes.push(format!("layoutOverride={}", override_id.as_str()));
            }
            Ok(SelectOutcome {
                note: notes.join(" "),
            })
        }
        Err(error) => {
            cache.invalidate();
            Err(error)
        }
    };
    Ok((
        current,
        outcome,
        SwitchGatewayTiming {
            current_micros,
            current_cache_hit,
            select_micros: micros_elapsed(select_started),
        },
    ))
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
            AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot, InputSourceId, LanguageTag,
            Platform, PlatformAppId, RuleTarget,
        },
    };

    const APP_ID: &str = "com.example.Editor";

    #[derive(Default)]
    struct GatewayData {
        current: String,
        selected: Vec<String>,
        select_attempts: usize,
        current_attempts: usize,
        refreshes: usize,
        fail_first_select: bool,
    }

    struct FakeGateway(Arc<Mutex<GatewayData>>);

    impl InputSourceGateway for FakeGateway {
        fn list(
            &mut self,
            refresh: bool,
            _locale: &str,
        ) -> Result<Vec<AvailableInputSource>, GatewayError> {
            if refresh {
                self.0.lock().expect("gateway lock").refreshes += 1;
            }
            Ok(Vec::new())
        }

        fn current(&self) -> Result<InputSourceId, GatewayError> {
            let mut data = self.0.lock().expect("gateway lock");
            data.current_attempts += 1;
            InputSourceId::parse(&data.current)
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

    fn snapshot_with_class(pid: i32, generation: u64, class: &str) -> FocusSnapshot {
        FocusSnapshot::new(
            app(pid),
            generation,
            vec![AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextArea")
                .with(AxAttribute::DomClassList, class)],
        )
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
        // 手動切替は OS 側の変更。通知が無い間は TTL か明示 invalidate で拾う。
        fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache lock")
            .invalidate();
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
    fn manual_override_latches_until_focus_key_changes() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 1, "composer-input"))
            .expect("auto switch");
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en"]
        );

        // ユーザーが手動で jp に変更（TIS 通知相当）。
        fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
        fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache lock")
            .invalidate();
        fixture.switcher.note_external_input_source_change();
        let latch_log = fixture
            .switcher
            .state()
            .logs
            .iter()
            .find(|log| log.reason.contains("latched as manual override"))
            .expect("tis latch log");
        assert!(!latch_log.focus_key.is_empty());
        assert!(latch_log.detail.contains("override=active"));

        // 同じ文脈・別 generation（偽のフォーカス変化）でも再 select しない。
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 2, "composer-input ime-input"))
            .expect("same focus key");
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en"]
        );
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            1
        );
        let skip_log = fixture
            .switcher
            .state()
            .logs
            .first()
            .expect("manual skip log");
        assert!(skip_log.reason.contains("manual input source change respected"));
        assert!(skip_log.detail.contains("override=active"));
        assert!(!skip_log.focus_key.is_empty());

        // 別フィールドへ移ったら通常どおり再適用。
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 3, "inputarea monaco"))
            .expect("new focus key");
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en", "en"]
        );
        let reapply_log = fixture
            .switcher
            .state()
            .logs
            .first()
            .expect("reapply log");
        assert!(reapply_log
            .reason
            .contains("override cleared (focus_key changed)"));
        assert!(reapply_log.reason.contains("selected target input source"));
    }

    #[test]
    fn own_select_tis_echo_does_not_latch_override() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 1, "composer-input"))
            .expect("auto switch");
        // select 成功後 current は en。エコー通知は override にしない。
        fixture.switcher.note_external_input_source_change();
        // override が無いので再評価は通るが、current==target で select は増えない。
        let decision = fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 2, "composer-input"))
            .expect("same key after echo");
        assert_eq!(decision, SwitchDecision::AlreadySelected);
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            1
        );
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en"]
        );
    }

    #[test]
    fn suppress_tis_as_manual_skips_override_latch() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 1, "composer-input"))
            .expect("auto switch");

        // 自前 select 中に current が一時的にずれても、suppress 中は override しない。
        fixture.switcher.set_suppress_tis_as_manual(true);
        fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
        fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache lock")
            .invalidate();
        fixture.switcher.note_external_input_source_change();
        fixture.switcher.set_suppress_tis_as_manual(false);

        // override が付いていないので、jp のまま再評価すると en へ select する。
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 2, "composer-input"))
            .expect("reapply without override");
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            2
        );
        assert_eq!(
            fixture.gateway.lock().expect("gateway lock").selected,
            ["en", "en"]
        );
    }

    #[test]
    fn activate_clears_manual_override() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 1, "composer-input"))
            .expect("auto switch");

        fixture.gateway.lock().expect("gateway lock").current = "jp".to_owned();
        fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache lock")
            .invalidate();
        fixture.switcher.note_external_input_source_change();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 2, "composer-input"))
            .expect("override holds");
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            1
        );

        // アプリ再前面化は override を捨てて再評価する。
        fixture
            .switcher
            .activate(app(42), 10)
            .expect("reactivate");
        fixture
            .switcher
            .current_cache_handle()
            .lock()
            .expect("cache lock")
            .invalidate();
        fixture
            .switcher
            .handle_focus(snapshot_with_class(42, 10, "composer-input"))
            .expect("reapply after activate");
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .select_attempts,
            2
        );
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
    fn same_source_skips_select() {
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
            0
        );
        let log = &fixture.switcher.state().logs[0];
        assert!(!log.attempted);
        assert!(log.reason.contains("select skipped"));
        assert!(!log.current_cache_hit);
    }

    #[test]
    fn current_source_cache_hits_within_ttl() {
        let mut fixture = Fixture::active();
        fixture.gateway.lock().expect("gateway lock").current = "en".to_owned();

        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("first focus");
        let attempts_after_miss = fixture
            .gateway
            .lock()
            .expect("gateway lock")
            .current_attempts;
        assert!(!fixture.switcher.state().logs[0].current_cache_hit);

        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("second focus");
        assert_eq!(
            fixture
                .gateway
                .lock()
                .expect("gateway lock")
                .current_attempts,
            attempts_after_miss
        );
        let log = &fixture.switcher.state().logs[0];
        assert!(log.current_cache_hit);
        assert_eq!(log.decision, SwitchDecision::AlreadySelected);
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
    fn switched_log_includes_post_select_verification() {
        let mut fixture = Fixture::active();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("switch");
        let log = &fixture.switcher.state().logs[0];
        assert_eq!(log.decision, SwitchDecision::Switched);
        assert!(log.reason.contains("postSelect=match"));
    }

    #[test]
    fn followup_reselect_log_only_when_debug_logging() {
        let mut fixture = Fixture::active();
        let id = InputSourceId::parse("en").expect("id");
        fixture.switcher.note_followup_reselect(40, 0, Some(&id));
        assert!(fixture.switcher.state().logs.is_empty());

        fixture
            .switcher
            .set_debug_logging(true)
            .expect("debug on");
        fixture.switcher.note_followup_reselect(40, 0, Some(&id));
        assert!(fixture
            .switcher
            .state()
            .logs
            .iter()
            .any(|log| log.reason.contains("followup reselect")));
    }

    #[test]
    fn focus_coalesced_is_logged() {
        let mut fixture = Fixture::active();
        fixture.switcher.note_focus_coalesced(2, 9);
        let log = &fixture.switcher.state().logs[0];
        assert!(log.reason.contains("coalesced 2"));
        assert!(log.reason.contains("kept gen=9"));
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

    #[test]
    fn duplicate_already_selected_skips_redundant_publish() {
        let mut fixture = Fixture::active();
        fixture.gateway.lock().expect("gateway lock").current = "en".to_owned();
        fixture
            .switcher
            .handle_focus(snapshot(42, 1))
            .expect("first alreadySelected");
        let after_first = fixture.versions.lock().expect("versions").len();
        fixture
            .switcher
            .handle_focus(snapshot(42, 2))
            .expect("second alreadySelected");
        let after_second = fixture.versions.lock().expect("versions").len();
        assert!(after_first >= 1);
        assert_eq!(
            after_second, after_first,
            "identical tray/FE state must not publish again"
        );
    }
}
