use super::*;
use crate::domain::input_source::{Platform, PlatformAppId};

fn snapshot(elements: Vec<AxElementSnapshot>) -> FocusSnapshot {
    FocusSnapshot::new(
        PlatformAppId::new(Platform::MacOs, "com.example.App", 1).expect("app"),
        1,
        elements,
    )
}

#[test]
fn prefers_dom_identifier_over_role() {
    let snap = snapshot(vec![AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextArea")
        .with(AxAttribute::DomIdentifier, "workbench.panel.aichat.input")]);
    let conditions = propose_conditions(&snap);
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].attribute(), AxAttribute::DomIdentifier);
    assert_eq!(conditions[0].operator(), MatchOperator::Contains);
    assert_eq!(conditions[0].value(), "workbench.panel.aichat.input");
    assert_eq!(conditions[0].scope(), SearchScope::FocusedOnly);
}

#[test]
fn uses_distinctive_class_token_when_id_missing() {
    let snap = snapshot(vec![AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextArea")
        .with(AxAttribute::DomClassList, "composer-input-blur-wrapper foo")]);
    let conditions = propose_conditions(&snap);
    assert!(conditions.iter().any(|condition| {
        condition.attribute() == AxAttribute::DomClassList
            && condition.value().contains("composer-input")
    }));
}

#[test]
fn falls_back_to_role_only() {
    let snap = snapshot(vec![
        AxElementSnapshot::new().with(AxAttribute::Role, "AXTextArea")
    ]);
    let conditions = propose_conditions(&snap);
    assert_eq!(conditions.len(), 1);
    assert_eq!(conditions[0].attribute(), AxAttribute::Role);
    assert_eq!(conditions[0].value(), "AXTextArea");
}

#[test]
fn field_label_prefers_description() {
    let snap = snapshot(vec![AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextArea")
        .with(AxAttribute::Description, "AI Chat")]);
    assert_eq!(propose_field_label(&snap).as_deref(), Some("AI Chat"));
}

#[test]
fn field_label_uses_placeholder_before_role_description() {
    let snap = snapshot(vec![AxElementSnapshot::new()
        .with(AxAttribute::Role, "AXTextField")
        .with(AxAttribute::Placeholder, "Search Google or type a URL")
        .with(AxAttribute::RoleDescription, "text field")]);
    assert_eq!(
        propose_field_label(&snap).as_deref(),
        Some("Search Google or type a URL")
    );
}

#[test]
fn field_label_is_none_when_ax_has_no_text() {
    let snap = snapshot(vec![
        AxElementSnapshot::new().with(AxAttribute::Role, "AXTextArea")
    ]);
    assert_eq!(propose_field_label(&snap), None);
    assert_eq!(propose_rule_name(&snap), RULE_NAME_FIELD_FALLBACK);
}
