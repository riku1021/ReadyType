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
