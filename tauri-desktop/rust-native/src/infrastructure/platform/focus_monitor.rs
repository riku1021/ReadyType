use std::collections::HashSet;

use crate::{
    application::ports::focus_monitor::{
        AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
    },
    domain::input_source::{
        focus_identity_key, AxAttribute, AxElementSnapshot, PlatformAppId, EMPTY_FOCUS_KEY,
        FOCUS_IDENTITY_DEPTH,
    },
};

pub const FALLBACK_INTERVAL_MS: u64 = 50;
pub const MAX_PARENT_DEPTH: usize = 14;
/// Description / Title など高コスト属性を取る祖先の深さ上限（0=focused のみ）。
/// ターミナル等の長い Description を祖先でも読むと AX が跳ねるため focused に限定する。
pub const EXPENSIVE_ATTRIBUTE_MAX_DEPTH: usize = 0;
/// `focus_fingerprint` が参照する祖先数。これ以降の Dom* / Identifier は
/// fingerprint 判定に不要なので第一パスでは読まない。
pub const FINGERPRINT_IDENTITY_DEPTH: usize = FOCUS_IDENTITY_DEPTH;

/// 切替ルールが付き得る focused Role（フル親ウォーク + enrich）。
/// 非候補は focused のみ・enrich なし（Button/Link 等の無駄 AX を避ける）。
pub fn is_switch_candidate_role(role: &str) -> bool {
    matches!(
        role,
        "AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField" | "AXGroup" | "AXWebArea"
    ) || role.to_ascii_lowercase().contains("text")
}

/// Description 等高コスト属性がルールに使われ得る Role。
pub fn role_may_need_description(role: &str) -> bool {
    matches!(
        role,
        "AXTextField" | "AXTextArea" | "AXComboBox" | "AXSearchField"
    ) || role.to_ascii_lowercase().contains("text")
}

/// Description / Title など、enrich で条件付きにする高コスト属性。
pub fn is_expensive_ax_attribute(attribute: AxAttribute) -> bool {
    matches!(
        attribute,
        AxAttribute::Description
            | AxAttribute::RoleDescription
            | AxAttribute::Title
            | AxAttribute::Help
            | AxAttribute::Placeholder
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObserverRegistration {
    Registered,
    Unsupported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitoringMode {
    Stopped,
    StoppedNoPermission,
    AxNotification,
    FallbackPolling { interval_ms: u64 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributePlan {
    attributes: Vec<AxAttribute>,
}

impl AttributePlan {
    pub fn normal(requested: impl IntoIterator<Item = AxAttribute>) -> Self {
        let mut seen = HashSet::new();
        let attributes = requested
            .into_iter()
            .filter(|attribute| seen.insert(*attribute))
            .collect();
        Self { attributes }
    }

    pub fn debug() -> Self {
        Self {
            attributes: vec![
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
            ],
        }
    }

    /// fingerprint 用の安い属性だけ（Description 等を含めない）。
    pub fn fingerprint() -> Self {
        Self {
            attributes: vec![
                AxAttribute::Role,
                AxAttribute::Identifier,
                AxAttribute::DomIdentifier,
                AxAttribute::DomClassList,
            ],
        }
    }

    pub fn is_fingerprint_attribute(attribute: &AxAttribute) -> bool {
        matches!(
            attribute,
            AxAttribute::Role
                | AxAttribute::Identifier
                | AxAttribute::DomIdentifier
                | AxAttribute::DomClassList
        )
    }

    /// fingerprint 以外（第二パスで足す属性）。
    pub fn attributes_beyond_fingerprint(&self) -> Vec<AxAttribute> {
        self.attributes
            .iter()
            .copied()
            .filter(|attribute| !Self::is_fingerprint_attribute(attribute))
            .collect()
    }

    pub fn attributes(&self) -> &[AxAttribute] {
        &self.attributes
    }
}

#[derive(Debug)]
pub struct MonitorState {
    permission_granted: bool,
    active_app: Option<PlatformAppId>,
    generation: u64,
    mode: MonitoringMode,
    pending_pid: Option<i32>,
    last_focus_fingerprint: Option<String>,
    restart_needed: bool,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            permission_granted: true,
            active_app: None,
            generation: 0,
            mode: MonitoringMode::Stopped,
            pending_pid: None,
            last_focus_fingerprint: None,
            restart_needed: false,
        }
    }
}

/// Stable-ish identity of the focused AX chain（domain の focus_identity_key に委譲）。
pub fn focus_fingerprint(elements: &[AxElementSnapshot]) -> String {
    focus_identity_key(elements)
}

impl MonitorState {
    pub fn activate(&mut self, app: PlatformAppId, registration: ObserverRegistration) {
        self.generation = self.generation.wrapping_add(1);
        self.active_app = Some(app);
        self.pending_pid = None;
        // App 再前面化では同一要素でもルール再評価したい。
        self.last_focus_fingerprint = None;
        self.restart_needed = false;
        self.mode = if !self.permission_granted {
            MonitoringMode::StoppedNoPermission
        } else {
            match registration {
                ObserverRegistration::Registered => MonitoringMode::AxNotification,
                ObserverRegistration::Unsupported => MonitoringMode::FallbackPolling {
                    interval_ms: FALLBACK_INTERVAL_MS,
                },
            }
        };
    }

    pub fn stop(&mut self) {
        self.active_app = None;
        self.pending_pid = None;
        self.last_focus_fingerprint = None;
        self.mode = MonitoringMode::Stopped;
    }

    pub fn update_permission(&mut self, granted: bool) {
        let restored = !self.permission_granted && granted;
        self.permission_granted = granted;
        if !granted {
            self.pending_pid = None;
            self.mode = MonitoringMode::StoppedNoPermission;
            self.restart_needed = false;
        } else if restored {
            self.restart_needed = true;
            self.mode = MonitoringMode::Stopped;
        }
    }

    pub fn enqueue(&mut self, pid: i32) -> bool {
        if !self.accepts_pid(pid) || self.pending_pid == Some(pid) {
            return false;
        }
        self.pending_pid = Some(pid);
        true
    }

    pub fn take_pending(&mut self) -> Option<i32> {
        self.pending_pid.take()
    }

    /// フォーカス指紋が変わったときだけ generation を進め、新規フォーカスとして返す。
    /// 空フォーカスは一時失敗とみなし generation を進めない（前回 identity を保持）。
    pub fn commit_focus_change(&mut self, fingerprint: &str) -> Option<u64> {
        if fingerprint == EMPTY_FOCUS_KEY {
            return None;
        }
        if self.last_focus_fingerprint.as_deref() == Some(fingerprint) {
            return None;
        }
        self.last_focus_fingerprint = Some(fingerprint.to_owned());
        self.generation = self.generation.wrapping_add(1);
        Some(self.generation)
    }

    /// 指紋が現在と同一か（コミットしない）。
    pub fn peek_focus_unchanged(&self, fingerprint: &str) -> bool {
        self.last_focus_fingerprint.as_deref() == Some(fingerprint)
    }

    pub fn accepts_pid(&self, pid: i32) -> bool {
        self.permission_granted
            && self.active_app.as_ref().is_some_and(|app| app.pid() == pid)
            && !matches!(
                self.mode,
                MonitoringMode::Stopped | MonitoringMode::StoppedNoPermission
            )
    }

    pub fn mode(&self) -> MonitoringMode {
        self.mode
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn needs_restart(&self) -> bool {
        self.restart_needed
    }

    pub fn active_identity(&self) -> Option<(i32, u64)> {
        self.active_app
            .as_ref()
            .map(|app| (app.pid(), self.generation))
    }

    pub fn active_app(&self) -> Option<&PlatformAppId> {
        self.active_app.as_ref()
    }
}

#[derive(Default)]
pub struct UnsupportedFocusMonitor;

impl FocusMonitor for UnsupportedFocusMonitor {
    fn start(
        &mut self,
        _attribute_index: crate::application::ports::focus_monitor::SwitchingAttributeIndex,
        _sink: FocusEventSink,
    ) -> Result<(), FocusMonitorError> {
        Err(FocusMonitorError::UnsupportedPlatform)
    }

    fn stop(&mut self) -> Result<(), FocusMonitorError> {
        Ok(())
    }

    fn set_debug_capture(&mut self, _enabled: bool) -> Result<(), FocusMonitorError> {
        Err(FocusMonitorError::UnsupportedPlatform)
    }

    fn permission_status(&self) -> AccessibilityPermission {
        AccessibilityPermission::Unsupported
    }

    fn request_permission(&self) -> Result<AccessibilityPermission, FocusMonitorError> {
        Err(FocusMonitorError::UnsupportedPlatform)
    }

    fn open_settings(&self) -> Result<(), FocusMonitorError> {
        Err(FocusMonitorError::UnsupportedPlatform)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        is_switch_candidate_role, role_may_need_description, AttributePlan, MonitorState,
        MonitoringMode, ObserverRegistration, EXPENSIVE_ATTRIBUTE_MAX_DEPTH, FALLBACK_INTERVAL_MS,
        FINGERPRINT_IDENTITY_DEPTH, MAX_PARENT_DEPTH,
    };
    use crate::domain::input_source::{
        AxAttribute, AxElementSnapshot, Platform, PlatformAppId, EMPTY_FOCUS_KEY,
    };

    fn app(id: &str, pid: i32) -> PlatformAppId {
        PlatformAppId::new(Platform::MacOs, id, pid).expect("valid app")
    }

    #[test]
    fn observer_failure_enables_fallback_only_for_frontmost_pid() {
        let mut state = MonitorState::default();
        state.activate(app("com.example", 41), ObserverRegistration::Unsupported);
        assert_eq!(
            state.mode(),
            MonitoringMode::FallbackPolling {
                interval_ms: FALLBACK_INTERVAL_MS
            }
        );

        state.activate(app("com.other", 42), ObserverRegistration::Registered);

        assert_eq!(state.mode(), MonitoringMode::AxNotification);
        assert!(!state.accepts_pid(41));
        assert!(state.accepts_pid(42));
    }

    #[test]
    fn permission_revocation_stops_observer_and_restoration_restarts() {
        let mut state = MonitorState::default();
        state.activate(app("com.example", 41), ObserverRegistration::Registered);

        state.update_permission(false);
        assert_eq!(state.mode(), MonitoringMode::StoppedNoPermission);
        assert!(!state.needs_restart());

        state.update_permission(true);
        assert_eq!(state.mode(), MonitoringMode::Stopped);
        assert!(state.needs_restart());
    }

    #[test]
    fn activation_increments_generation_and_duplicate_signals_are_coalesced() {
        let mut state = MonitorState::default();
        state.activate(app("com.example", 41), ObserverRegistration::Registered);
        let generation = state.generation();
        assert!(state.enqueue(41));
        assert!(!state.enqueue(41));
        assert_eq!(state.take_pending(), Some(41));

        state.activate(app("com.other", 42), ObserverRegistration::Registered);
        assert!(state.generation() > generation);
        assert!(!state.enqueue(41));
    }

    #[test]
    fn focus_fingerprint_change_advances_generation_once_per_identity() {
        let mut state = MonitorState::default();
        state.activate(app("com.example", 41), ObserverRegistration::Registered);
        let after_activate = state.generation();

        let first = state
            .commit_focus_change("textarea`aichat``")
            .expect("first focus");
        assert!(first > after_activate);
        assert_eq!(state.commit_focus_change("textarea`aichat``"), None);

        let second = state
            .commit_focus_change("textarea`editor``")
            .expect("second focus");
        assert!(second > first);
    }

    #[test]
    fn focus_fingerprint_ignores_description_volatility() {
        let left = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomClassList, "aislash-editor-input")
            .with(AxAttribute::Description, "message A")];
        let right = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomClassList, "aislash-editor-input")
            .with(AxAttribute::Description, "message B")];
        assert_eq!(
            super::focus_fingerprint(&left),
            super::focus_fingerprint(&right)
        );
    }

    #[test]
    fn focus_fingerprint_ignores_volatile_dom_class_tokens() {
        let base = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(
                AxAttribute::DomClassList,
                "inputarea monaco-mouse-cursor-text",
            )];
        let with_ime = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(
                AxAttribute::DomClassList,
                "inputarea monaco-mouse-cursor-text ime-input",
            )];
        assert_eq!(
            super::focus_fingerprint(&base),
            super::focus_fingerprint(&with_ime)
        );
    }

    #[test]
    fn focus_fingerprint_ignores_ancestor_dom_class_churn() {
        let stable = [
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextField")
                .with(AxAttribute::DomClassList, "xterm-helper-textarea"),
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXGroup")
                .with(AxAttribute::DomClassList, "split-view-view"),
        ];
        let churned = [
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextField")
                .with(AxAttribute::DomClassList, "xterm-helper-textarea"),
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXGroup")
                .with(AxAttribute::DomClassList, "split-view-view visible"),
        ];
        assert_eq!(
            super::focus_fingerprint(&stable),
            super::focus_fingerprint(&churned)
        );
    }

    #[test]
    fn empty_fingerprint_does_not_advance_generation() {
        let mut state = MonitorState::default();
        state.activate(app("com.example", 41), ObserverRegistration::Registered);
        let after_activate = state.generation();
        let first = state
            .commit_focus_change("textarea`aichat``")
            .expect("first focus");
        assert!(first > after_activate);
        assert_eq!(state.commit_focus_change(EMPTY_FOCUS_KEY), None);
        assert_eq!(state.commit_focus_change("textarea`aichat``"), None);
    }

    #[test]
    fn attribute_plans_are_minimal_normally_and_complete_for_debugging() {
        let normal = AttributePlan::normal([
            AxAttribute::Role,
            AxAttribute::DomIdentifier,
            AxAttribute::Role,
        ]);
        assert_eq!(
            normal.attributes(),
            &[AxAttribute::Role, AxAttribute::DomIdentifier]
        );

        let debug = AttributePlan::debug();
        assert_eq!(debug.attributes().len(), 10);
        assert!(debug.attributes().contains(&AxAttribute::Help));
        assert!(debug.attributes().contains(&AxAttribute::Placeholder));
        assert!(debug.attributes().contains(&AxAttribute::DomClassList));
    }

    #[test]
    fn fingerprint_plan_excludes_description() {
        let plan = AttributePlan::fingerprint();
        assert!(plan.attributes().contains(&AxAttribute::Role));
        assert!(plan.attributes().contains(&AxAttribute::DomClassList));
        assert!(!plan.attributes().contains(&AxAttribute::Description));
    }

    #[test]
    fn peek_focus_unchanged_matches_last_fingerprint() {
        let mut state = MonitorState::default();
        assert!(!state.peek_focus_unchanged("a"));
        assert_eq!(state.commit_focus_change("a"), Some(1));
        assert!(state.peek_focus_unchanged("a"));
        assert!(!state.peek_focus_unchanged("b"));
    }

    #[test]
    fn parent_walk_is_bounded() {
        assert_eq!(MAX_PARENT_DEPTH, 14);
        assert_eq!(EXPENSIVE_ATTRIBUTE_MAX_DEPTH, 0);
        assert_eq!(FINGERPRINT_IDENTITY_DEPTH, 6);
    }

    #[test]
    fn attributes_beyond_fingerprint_excludes_identity_attrs() {
        let plan = AttributePlan::normal([
            AxAttribute::Role,
            AxAttribute::DomClassList,
            AxAttribute::Description,
            AxAttribute::Placeholder,
        ]);
        let extra = plan.attributes_beyond_fingerprint();
        assert_eq!(
            extra,
            vec![AxAttribute::Description, AxAttribute::Placeholder]
        );
    }

    #[test]
    fn switch_candidate_role_covers_text_and_group() {
        let cases = [
            ("AXTextField", true),
            ("AXTextArea", true),
            ("AXComboBox", true),
            ("AXSearchField", true),
            ("AXGroup", true),
            ("AXWebArea", true),
            ("AXStaticText", true),
            ("AXButton", false),
            ("AXLink", false),
            ("AXRow", false),
            ("AXImage", false),
            ("AXTab", false),
            ("", false),
        ];
        for (role, expected) in cases {
            assert_eq!(
                is_switch_candidate_role(role),
                expected,
                "role={role:?} expected={expected}"
            );
        }
    }

    #[test]
    fn role_may_need_description_excludes_group() {
        assert!(role_may_need_description("AXTextField"));
        assert!(role_may_need_description("AXTextArea"));
        assert!(role_may_need_description("AXStaticText"));
        assert!(!role_may_need_description("AXGroup"));
        assert!(!role_may_need_description("AXWebArea"));
        assert!(!role_may_need_description("AXButton"));
    }
}
