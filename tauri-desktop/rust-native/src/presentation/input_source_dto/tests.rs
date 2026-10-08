use super::{RuleDto, StateDto};
use crate::{
    application::switcher_state::SwitcherState,
    domain::input_source::{Platform, PlatformAppId},
};

#[test]
fn dto_uses_camel_case_and_preserves_state_version() {
    let mut state = SwitcherState::new(true, None);
    state.version = 9;
    state.current_app =
        Some(PlatformAppId::new(Platform::MacOs, "com.example", 42).expect("valid app"));
    let json = serde_json::to_value(StateDto::from(&state)).expect("serialize");
    assert_eq!(json["version"], 9);
    assert!(json.get("currentApp").is_some());
    assert!(json.get("current_app").is_none());
}

#[test]
fn invalid_rule_dto_is_rejected() {
    let json = serde_json::json!({
        "id": "rule",
        "name": "Rule",
        "enabled": true,
        "platform": "macOs",
        "appId": "com.example",
        "priority": 0,
        "conditions": [{
            "attribute": "role",
            "operator": "equals",
            "scope": "focusedOnly",
            "value": " "
        }],
        "target": { "kind": "preferredLanguage", "language": "en" }
    });
    let dto: RuleDto = serde_json::from_value(json).expect("wire shape");
    assert!(crate::domain::input_source::FocusRule::try_from(dto).is_err());
}

#[test]
fn exact_target_accepts_camel_case_input_source_id() {
    let input_source_id = "com.example.inputsource.any-language";
    let json = serde_json::json!({
        "id": "rule",
        "name": "Rule",
        "enabled": true,
        "platform": "macOs",
        "appId": "com.example",
        "priority": 0,
        "conditions": [],
        "target": {
            "kind": "exact",
            "inputSourceId": input_source_id
        }
    });
    let dto: RuleDto = serde_json::from_value(json).expect("wire shape");
    let rule = crate::domain::input_source::FocusRule::try_from(dto).expect("valid rule");
    assert_eq!(
        rule.target(),
        &crate::domain::input_source::RuleTarget::Exact(
            crate::domain::input_source::InputSourceId::parse(input_source_id).expect("valid id")
        )
    );
}
