//! Quick Set session: Level-1 hotkey popover data and apply logic helpers.

use crate::domain::input_source::{
    propose_conditions, propose_rule_name, FocusRule, FocusSnapshot, InputSourceId, LanguageTag,
    Platform, RuleCondition, RuleTarget,
};

use super::ports::input_source::AvailableInputSource;
use super::settings::SwitcherSettings;

#[derive(Debug, Clone, PartialEq)]
pub struct QuickSetSession {
    pub app_id: String,
    pub app_display_name: String,
    pub platform: Platform,
    pub pid: i32,
    pub proposed_name: String,
    pub proposed_conditions: Vec<RuleCondition>,
    /// Present only when an existing *contextual* (non-empty conditions) rule matched.
    pub matched_rule_id: Option<String>,
    pub matched_target: Option<RuleTarget>,
    /// Same input-source list the app-settings LanguageSelect uses (OS snapshot).
    pub sources: Vec<AvailableInputSource>,
    pub language_preferences: std::collections::BTreeMap<LanguageTag, Option<InputSourceId>>,
}

pub fn build_quick_set_session(
    snapshot: &FocusSnapshot,
    rules: &[FocusRule],
    settings: &SwitcherSettings,
    sources: &[AvailableInputSource],
    // Finder-style OS name when available; otherwise falls back to a short bundle id slug.
    app_display_name: Option<String>,
) -> QuickSetSession {
    let app_id = snapshot.app().app_id().to_owned();
    let proposed_conditions = propose_conditions(snapshot);

    let matched = rules
        .iter()
        .filter(|rule| rule.matches(snapshot))
        .min_by_key(|rule| rule.priority());

    let (matched_rule_id, matched_target, proposed_name) = match matched {
        Some(rule) if !rule.conditions().is_empty() => (
            Some(rule.id().to_owned()),
            Some(rule.target().clone()),
            // Existing rule name is the user's truth when updating.
            rule.name().to_owned(),
        ),
        _ => (None, None, propose_rule_name(snapshot)),
    };

    let app_display_name = app_display_name
        .map(|name| name.trim().to_owned())
        .filter(|name| !name.is_empty())
        .unwrap_or_else(|| short_app_display_name(&app_id));

    QuickSetSession {
        app_id: app_id.clone(),
        app_display_name,
        platform: snapshot.app().platform(),
        pid: snapshot.app().pid(),
        proposed_name,
        proposed_conditions,
        matched_rule_id,
        matched_target,
        sources: sources.to_vec(),
        language_preferences: settings.language_preferences.clone(),
    }
}

/// Map a chosen input source to a rule target the same way the app-settings UI does:
/// preference hit → PreferredLanguage, otherwise Exact.
pub fn rule_target_from_source_id(
    source_id: &InputSourceId,
    settings: &SwitcherSettings,
) -> RuleTarget {
    for (tag, preferred) in &settings.language_preferences {
        if preferred.as_ref() == Some(source_id) {
            return RuleTarget::PreferredLanguage(tag.clone());
        }
    }
    RuleTarget::Exact(source_id.clone())
}

pub fn next_quick_set_priority(rules: &[FocusRule], app_id: &str) -> u32 {
    let min_contextual = rules
        .iter()
        .filter(|rule| rule.app_id() == app_id && !rule.conditions().is_empty())
        .map(FocusRule::priority)
        .min();
    match min_contextual {
        Some(0) => 0,
        Some(priority) => priority.saturating_sub(1),
        None => 50,
    }
}

/// Resolve the persisted rule name: non-empty user input wins, else session draft.
pub fn resolve_quick_set_name(session: &QuickSetSession, name: &str) -> String {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        session.proposed_name.clone()
    } else {
        trimmed.to_owned()
    }
}

pub fn build_created_rule(
    session: &QuickSetSession,
    target: RuleTarget,
    priority: u32,
    rule_id: &str,
    name: &str,
) -> Result<FocusRule, crate::domain::errors::DomainError> {
    FocusRule::new(
        rule_id,
        name,
        true,
        session.platform,
        &session.app_id,
        priority,
        session.proposed_conditions.clone(),
        target,
    )
}

pub fn build_updated_rule(
    existing: &FocusRule,
    target: RuleTarget,
    name: &str,
) -> Result<FocusRule, crate::domain::errors::DomainError> {
    FocusRule::new(
        existing.id(),
        name,
        existing.enabled(),
        existing.platform(),
        existing.app_id(),
        existing.priority(),
        existing.conditions().to_vec(),
        target,
    )
}

fn short_app_display_name(app_id: &str) -> String {
    app_id
        .rsplit('.')
        .next()
        .filter(|part| !part.is_empty())
        .unwrap_or(app_id)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::input_source::{
        AxAttribute, AxElementSnapshot, MatchOperator, PlatformAppId, SearchScope,
    };

    fn snap(elements: Vec<AxElementSnapshot>) -> FocusSnapshot {
        FocusSnapshot::new(
            PlatformAppId::new(Platform::MacOs, "com.todesktop.cursor", 42).expect("app"),
            1,
            elements,
        )
    }

    fn source(id: &str, name: &str, language: &str) -> AvailableInputSource {
        AvailableInputSource {
            id: InputSourceId::parse(id).expect("id"),
            name: name.to_owned(),
            language_code: Some(language.to_owned()),
            available: true,
        }
    }

    #[test]
    fn skips_app_wide_match_for_update() {
        let snapshot = snap(vec![AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomIdentifier, "aichat.input")]);
        let app_wide = FocusRule::new(
            "app-default",
            "Cursor",
            true,
            Platform::MacOs,
            "com.todesktop.cursor",
            200,
            Vec::new(),
            RuleTarget::PreferredLanguage(LanguageTag::en()),
        )
        .expect("rule");
        let settings = SwitcherSettings::defaults();
        let sources = vec![source("com.apple.keylayout.ABC", "ABC", "en")];
        let session = build_quick_set_session(&snapshot, &[app_wide], &settings, &sources, None);
        assert!(session.matched_rule_id.is_none());
        assert!(!session.proposed_conditions.is_empty());
        assert_eq!(session.sources.len(), 1);
    }

    #[test]
    fn keeps_contextual_match_for_update() {
        let snapshot = snap(vec![AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomIdentifier, "aichat.input")]);
        let condition = RuleCondition::new(
            AxAttribute::DomIdentifier,
            MatchOperator::Contains,
            SearchScope::FocusedOnly,
            "aichat.input",
        )
        .expect("condition");
        let contextual = FocusRule::new(
            "field-rule",
            "AI Chat",
            true,
            Platform::MacOs,
            "com.todesktop.cursor",
            100,
            vec![condition],
            RuleTarget::PreferredLanguage(LanguageTag::ja()),
        )
        .expect("rule");
        let settings = SwitcherSettings::defaults();
        let session = build_quick_set_session(
            &snapshot,
            &[contextual],
            &settings,
            &[],
            Some("Cursor".to_owned()),
        );
        assert_eq!(session.matched_rule_id.as_deref(), Some("field-rule"));
        assert_eq!(session.app_display_name, "Cursor");
        assert_eq!(session.proposed_name, "AI Chat");
    }

    #[test]
    fn resolve_quick_set_name_prefers_user_input() {
        let session = QuickSetSession {
            app_id: "com.example.app".to_owned(),
            app_display_name: "Example".to_owned(),
            platform: Platform::MacOs,
            pid: 1,
            proposed_name: "draft".to_owned(),
            proposed_conditions: Vec::new(),
            matched_rule_id: None,
            matched_target: None,
            sources: Vec::new(),
            language_preferences: Default::default(),
        };
        assert_eq!(resolve_quick_set_name(&session, "  URL bar  "), "URL bar");
        assert_eq!(resolve_quick_set_name(&session, "   "), "draft");
        assert_eq!(resolve_quick_set_name(&session, ""), "draft");
    }

    #[test]
    fn rule_target_from_source_id_prefers_preference_then_exact() {
        let ja = InputSourceId::parse("com.apple.inputmethod.Kotoeri.Japanese").expect("id");
        let other = InputSourceId::parse("com.apple.keylayout.ABC").expect("id");
        let mut settings = SwitcherSettings::defaults();
        settings
            .language_preferences
            .insert(LanguageTag::ja(), Some(ja.clone()));

        assert_eq!(
            rule_target_from_source_id(&ja, &settings),
            RuleTarget::PreferredLanguage(LanguageTag::ja())
        );
        assert_eq!(
            rule_target_from_source_id(&other, &settings),
            RuleTarget::Exact(other)
        );
    }
}
