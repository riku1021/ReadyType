use super::default_rules;
use crate::domain::input_source::{
    AxAttribute, AxElementSnapshot, FocusSnapshot, Platform, PlatformAppId, RuleSet,
};

fn cursor_snapshot(elements: Vec<AxElementSnapshot>) -> FocusSnapshot {
    FocusSnapshot::new(
        PlatformAppId::new(Platform::MacOs, "com.todesktop.230313mzl4w4u92", 42)
            .expect("valid app"),
        1,
        elements,
    )
}

fn element(values: &[(AxAttribute, &str)]) -> AxElementSnapshot {
    values
        .iter()
        .fold(AxElementSnapshot::new(), |element, (attribute, value)| {
            element.with(*attribute, *value)
        })
}

fn matched_rule_name(snapshot: &FocusSnapshot) -> Option<String> {
    let rules = RuleSet::new(default_rules()).expect("valid presets");
    rules
        .first_match(snapshot)
        .map(|rule| rule.name().to_owned())
}

#[test]
fn cursor_terminal_wins_before_chat() {
    let snapshot = cursor_snapshot(vec![element(&[
        (AxAttribute::Role, "AXTextArea"),
        (AxAttribute::Description, "Terminal input"),
        (AxAttribute::DomIdentifier, "workbench.panel.aichat"),
    ])]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Terminal")
    );
}

#[test]
fn cursor_chat_wins_before_editor() {
    let snapshot = cursor_snapshot(vec![
        element(&[(AxAttribute::Role, "AXTextArea")]),
        element(&[(AxAttribute::DomIdentifier, "workbench.panel.aichat.input")]),
        element(&[(AxAttribute::DomIdentifier, "workbench.parts.editor")]),
    ]);

    assert_eq!(matched_rule_name(&snapshot).as_deref(), Some("Cursor Chat"));
}

#[test]
fn cursor_composer_class_list_matches_preferred_japanese_rule() {
    let snapshot = cursor_snapshot(vec![
        element(&[
            (AxAttribute::Role, "AXTextArea"),
            (AxAttribute::DomClassList, "aislash-editor-input"),
        ]),
        element(&[(AxAttribute::DomClassList, "composer-input-blur-wrapper")]),
        element(&[(AxAttribute::DomClassList, "composer-bar editor")]),
        element(&[(AxAttribute::DomIdentifier, "workbench.parts.editor")]),
    ]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Composer")
    );
}

#[test]
fn cursor_ai_input_class_list_matches_when_composer_prefix_absent() {
    let snapshot = cursor_snapshot(vec![element(&[
        (AxAttribute::Role, "AXTextArea"),
        (AxAttribute::DomClassList, "aislash-editor-input"),
    ])]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor AI Input")
    );
}

#[test]
fn cursor_rename_wins_before_editor() {
    let snapshot = cursor_snapshot(vec![
        element(&[
            (AxAttribute::Role, "AXTextField"),
            (
                AxAttribute::Description,
                "Press Enter to confirm the file name",
            ),
        ]),
        element(&[(AxAttribute::DomIdentifier, "workbench.parts.editor")]),
    ]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Rename")
    );
}

#[test]
fn cursor_japanese_rename_description_is_supported() {
    let snapshot = cursor_snapshot(vec![element(&[
        (AxAttribute::Role, "AXTextField"),
        (AxAttribute::Description, "ファイル名を入力します"),
    ])]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Rename")
    );
}

#[test]
fn cursor_japanese_terminal_description_is_supported() {
    let snapshot = cursor_snapshot(vec![element(&[
        (AxAttribute::Role, "AXTextArea"),
        (AxAttribute::Description, "統合ターミナル"),
    ])]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Terminal")
    );
}

#[test]
fn vscode_chat_dom_identifier_is_supported() {
    let snapshot = FocusSnapshot::new(
        PlatformAppId::new(Platform::MacOs, "com.microsoft.VSCode", 99).expect("valid app"),
        1,
        vec![element(&[(
            AxAttribute::DomIdentifier,
            "workbench.panel.chat.input",
        )])],
    );

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("VS Code Chat")
    );
}

#[test]
fn cursor_editor_matches_after_higher_priority_contexts_fail() {
    let snapshot = cursor_snapshot(vec![element(&[(
        AxAttribute::DomIdentifier,
        "workbench.parts.editor",
    )])]);

    assert_eq!(
        matched_rule_name(&snapshot).as_deref(),
        Some("Cursor Editor")
    );
}

#[test]
fn terminal_rule_matches_without_ax_elements() {
    let snapshot = FocusSnapshot::new(
        PlatformAppId::new(Platform::MacOs, "com.apple.Terminal", 7).expect("valid app"),
        1,
        Vec::new(),
    );

    assert_eq!(matched_rule_name(&snapshot).as_deref(), Some("Terminal"));
}

#[test]
fn preset_ids_are_stable_and_unique() {
    let rules = default_rules();
    let ids = rules
        .iter()
        .map(|rule| rule.id().to_owned())
        .collect::<Vec<_>>();

    assert!(ids.iter().any(|id| id == "preset-terminal"));
    assert!(ids.iter().any(|id| id == "preset-cursor-chat"));
    assert!(ids.iter().any(|id| id == "preset-cursor-composer"));
    assert!(ids.iter().any(|id| id == "preset-cursor-ai-input"));
    assert!(ids.iter().any(|id| id == "preset-vscode-editor"));
    assert!(ids.iter().any(|id| id == "preset-readytype-lab-switch-01"));
    assert!(ids.iter().any(|id| id == "preset-readytype-lab-switch-06"));
    assert_eq!(
        RuleSet::new(rules).expect("unique presets").rules().len(),
        ids.len()
    );
}

#[test]
fn readytype_lab_switch_fields_match_fixed_languages() {
    let app = PlatformAppId::new(Platform::MacOs, "com.readytype.app", 1).expect("valid app");
    let rules = RuleSet::new(default_rules()).expect("valid presets");

    let cases = [
        ("readytype-lab-switch-01", "preset-readytype-lab-switch-01"),
        ("readytype-lab-switch-02", "preset-readytype-lab-switch-02"),
        ("readytype-lab-switch-06", "preset-readytype-lab-switch-06"),
    ];
    for (identifier, expected_rule_id) in cases {
        let snapshot = FocusSnapshot::new(
            app.clone(),
            1,
            vec![element(&[(AxAttribute::Identifier, identifier)])],
        );
        let matched = rules.first_match(&snapshot).expect("lab field matches");
        assert_eq!(matched.id(), expected_rule_id);

        // field editor が FOCUSED で Identifier は親 NSTextField にあるケース
        let via_ancestor = FocusSnapshot::new(
            app.clone(),
            1,
            vec![
                element(&[(AxAttribute::Role, "AXTextArea")]),
                element(&[(AxAttribute::Identifier, identifier)]),
            ],
        );
        let matched = rules
            .first_match(&via_ancestor)
            .expect("lab field matches via ancestor");
        assert_eq!(matched.id(), expected_rule_id);
    }
}
