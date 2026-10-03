//! Quick Set session: Level-1 hotkey popover data and apply logic helpers.

use crate::domain::input_source::{
    propose_conditions, propose_field_label, propose_rule_name, FocusRule, FocusSnapshot,
    LanguageTag, Platform, RuleCondition, RuleTarget,
};

use super::ports::input_source::AvailableInputSource;
use super::settings::SwitcherSettings;

#[derive(Debug, Clone, PartialEq)]
pub struct QuickSetLanguageOption {
    pub language: LanguageTag,
    pub preferred_source_id: String,
    pub preferred_source_name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct QuickSetSession {
    pub app_id: String,
    pub app_display_name: String,
    pub platform: Platform,
    pub pid: i32,
    pub field_label: String,
    pub proposed_name: String,
    pub proposed_conditions: Vec<RuleCondition>,
    /// Present only when an existing *contextual* (non-empty conditions) rule matched.
    pub matched_rule_id: Option<String>,
    pub matched_target: Option<RuleTarget>,
    pub languages: Vec<QuickSetLanguageOption>,
}

pub fn build_quick_set_session(
    snapshot: &FocusSnapshot,
    rules: &[FocusRule],
    settings: &SwitcherSettings,
    sources: &[AvailableInputSource],
) -> QuickSetSession {
    let app_id = snapshot.app().app_id().to_owned();
    // Empty when AX has no label; FE localizes the display fallback.
    let field_label = propose_field_label(snapshot).unwrap_or_default();
    let proposed_name = propose_rule_name(snapshot);
    let proposed_conditions = propose_conditions(snapshot);

    let matched = rules
        .iter()
        .filter(|rule| rule.matches(snapshot))
        .min_by_key(|rule| rule.priority());

    let (matched_rule_id, matched_target) = match matched {
        Some(rule) if !rule.conditions().is_empty() => {
            (Some(rule.id().to_owned()), Some(rule.target().clone()))
        }
        _ => (None, None),
    };

    let languages = settings
        .language_preferences
        .iter()
        .filter_map(|(tag, preferred)| {
            let source_id = preferred.as_ref()?;
            let name = sources
                .iter()
                .find(|source| source.id.as_str() == source_id.as_str())
                .map(|source| source.name.clone())
                .unwrap_or_else(|| source_id.as_str().to_owned());
            Some(QuickSetLanguageOption {
                language: tag.clone(),
                preferred_source_id: source_id.as_str().to_owned(),
                preferred_source_name: name,
            })
        })
        .collect();

    QuickSetSession {
        app_id: app_id.clone(),
        app_display_name: short_app_display_name(&app_id),
        platform: snapshot.app().platform(),
        pid: snapshot.app().pid(),
        field_label,
        proposed_name,
        proposed_conditions,
        matched_rule_id,
        matched_target,
        languages,
    }
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

pub fn build_created_rule(
    session: &QuickSetSession,
    language: LanguageTag,
    priority: u32,
    rule_id: &str,
) -> Result<FocusRule, crate::domain::errors::DomainError> {
    FocusRule::new(
        rule_id,
        &session.proposed_name,
        true,
        session.platform,
        &session.app_id,
        priority,
        session.proposed_conditions.clone(),
        RuleTarget::PreferredLanguage(language),
    )
}

pub fn build_updated_rule(
    existing: &FocusRule,
    language: LanguageTag,
) -> Result<FocusRule, crate::domain::errors::DomainError> {
    FocusRule::new(
        existing.id(),
        existing.name(),
        existing.enabled(),
        existing.platform(),
        existing.app_id(),
        existing.priority(),
        existing.conditions().to_vec(),
        RuleTarget::PreferredLanguage(language),
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
        AxAttribute, AxElementSnapshot, InputSourceId, MatchOperator, PlatformAppId, SearchScope,
    };

    fn snap(elements: Vec<AxElementSnapshot>) -> FocusSnapshot {
        FocusSnapshot::new(
            PlatformAppId::new(Platform::MacOs, "com.todesktop.cursor", 42).expect("app"),
            1,
            elements,
        )
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
        let mut settings = SwitcherSettings::defaults();
        settings.language_preferences.insert(
            LanguageTag::ja(),
            Some(InputSourceId::parse("com.apple.inputmethod.Kotoeri.Japanese").expect("id")),
        );
        let session = build_quick_set_session(&snapshot, &[app_wide], &settings, &[]);
        assert!(session.matched_rule_id.is_none());
        assert!(!session.proposed_conditions.is_empty());
        assert_eq!(session.languages.len(), 1);
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
        let session = build_quick_set_session(&snapshot, &[contextual], &settings, &[]);
        assert_eq!(session.matched_rule_id.as_deref(), Some("field-rule"));
    }
}
