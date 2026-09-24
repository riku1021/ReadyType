use std::collections::HashSet;

use crate::{
    application::ports::focus_monitor::{
        AccessibilityPermission, FocusEventSink, FocusMonitor, FocusMonitorError,
    },
    domain::input_source::{AxAttribute, PlatformAppId},
};

pub const FALLBACK_INTERVAL_MS: u64 = 50;
pub const MAX_PARENT_DEPTH: usize = 14;

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
                AxAttribute::Help,
            ],
        }
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
    pending_signal: Option<(i32, u64)>,
    restart_needed: bool,
}

impl Default for MonitorState {
    fn default() -> Self {
        Self {
            permission_granted: true,
            active_app: None,
            generation: 0,
            mode: MonitoringMode::Stopped,
            pending_signal: None,
            restart_needed: false,
        }
    }
}

impl MonitorState {
    pub fn activate(&mut self, app: PlatformAppId, registration: ObserverRegistration) {
        self.generation = self.generation.wrapping_add(1);
        self.active_app = Some(app);
        self.pending_signal = None;
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
        self.pending_signal = None;
        self.mode = MonitoringMode::Stopped;
    }

    pub fn update_permission(&mut self, granted: bool) {
        let restored = !self.permission_granted && granted;
        self.permission_granted = granted;
        if !granted {
            self.pending_signal = None;
            self.mode = MonitoringMode::StoppedNoPermission;
            self.restart_needed = false;
        } else if restored {
            self.restart_needed = true;
            self.mode = MonitoringMode::Stopped;
        }
    }

    pub fn enqueue(&mut self, pid: i32, generation: u64) -> bool {
        if !self.accepts(pid, generation) || self.pending_signal == Some((pid, generation)) {
            return false;
        }
        self.pending_signal = Some((pid, generation));
        true
    }

    pub fn take_pending(&mut self) -> Option<(i32, u64)> {
        self.pending_signal.take()
    }

    pub fn accepts(&self, pid: i32, generation: u64) -> bool {
        self.permission_granted
            && self.generation == generation
            && self.active_app.as_ref().is_some_and(|app| app.pid() == pid)
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
        _attributes: Vec<AxAttribute>,
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
        AttributePlan, MonitorState, MonitoringMode, ObserverRegistration, FALLBACK_INTERVAL_MS,
        MAX_PARENT_DEPTH,
    };
    use crate::domain::input_source::{AxAttribute, Platform, PlatformAppId};

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
        let old_generation = state.generation();

        state.activate(app("com.other", 42), ObserverRegistration::Registered);

        assert_eq!(state.mode(), MonitoringMode::AxNotification);
        assert!(!state.accepts(41, old_generation));
        assert!(state.accepts(42, state.generation()));
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
        assert!(state.enqueue(41, generation));
        assert!(!state.enqueue(41, generation));
        assert_eq!(state.take_pending(), Some((41, generation)));

        state.activate(app("com.other", 42), ObserverRegistration::Registered);
        assert!(state.generation() > generation);
        assert!(!state.enqueue(41, generation));
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
        assert_eq!(debug.attributes().len(), 8);
        assert!(debug.attributes().contains(&AxAttribute::Help));
    }

    #[test]
    fn parent_walk_is_bounded() {
        assert_eq!(MAX_PARENT_DEPTH, 14);
    }
}
