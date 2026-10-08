use super::{
    AxAttribute, FocusRule, LanguageTag, MatchOperator, Platform, RuleCondition, RuleTarget,
    SearchScope,
};

fn preferred_en() -> RuleTarget {
    RuleTarget::PreferredLanguage(LanguageTag::en())
}

fn preferred_ja() -> RuleTarget {
    RuleTarget::PreferredLanguage(LanguageTag::ja())
}

const TERMINAL_BUNDLE_ID: &str = "com.apple.Terminal";
const CURSOR_BUNDLE_ID: &str = "com.todesktop.230313mzl4w4u92";
const VSCODE_BUNDLE_ID: &str = "com.microsoft.VSCode";
pub const READYTYPE_BUNDLE_ID: &str = "com.readytype.app";

/// Lab 切替テスト欄の AXIdentifier（ネイティブ NSTextField の accessibilityIdentifier と一致）。
pub const LAB_SWITCH_FIELD_IDS: [&str; 6] = [
    "readytype-lab-switch-01",
    "readytype-lab-switch-02",
    "readytype-lab-switch-03",
    "readytype-lab-switch-04",
    "readytype-lab-switch-05",
    "readytype-lab-switch-06",
];

fn condition(attribute: AxAttribute, operator: MatchOperator, value: &str) -> RuleCondition {
    RuleCondition::new(attribute, operator, SearchScope::FocusedAndAncestors, value)
        .expect("default condition must be valid")
}

fn rule(
    id: &str,
    name: &str,
    app_id: &str,
    priority: u32,
    conditions: Vec<RuleCondition>,
    target: RuleTarget,
) -> FocusRule {
    FocusRule::new(
        id,
        name,
        true,
        Platform::MacOs,
        app_id,
        priority,
        conditions,
        target,
    )
    .expect("default rule must be valid")
}

fn contextual_rules(prefix: &str, app_name: &str, app_id: &str) -> Vec<FocusRule> {
    vec![
        rule(
            &format!("preset-{prefix}-terminal-en"),
            &format!("{app_name} Terminal"),
            app_id,
            100,
            vec![
                condition(AxAttribute::Role, MatchOperator::Contains, "text"),
                condition(
                    AxAttribute::Description,
                    MatchOperator::Contains,
                    "terminal",
                ),
            ],
            preferred_en(),
        ),
        rule(
            &format!("preset-{prefix}-terminal-ja"),
            &format!("{app_name} Terminal"),
            app_id,
            101,
            vec![
                condition(AxAttribute::Role, MatchOperator::Contains, "text"),
                condition(
                    AxAttribute::Description,
                    MatchOperator::Contains,
                    "ターミナル",
                ),
            ],
            preferred_en(),
        ),
        rule(
            &format!("preset-{prefix}-rename-en"),
            &format!("{app_name} Rename"),
            app_id,
            120,
            vec![
                condition(AxAttribute::Role, MatchOperator::Equals, "AXTextField"),
                condition(
                    AxAttribute::Description,
                    MatchOperator::Contains,
                    "press enter to confirm",
                ),
            ],
            preferred_en(),
        ),
        rule(
            &format!("preset-{prefix}-rename-ja"),
            &format!("{app_name} Rename"),
            app_id,
            121,
            vec![
                condition(AxAttribute::Role, MatchOperator::Equals, "AXTextField"),
                condition(
                    AxAttribute::Description,
                    MatchOperator::Contains,
                    "ファイル名を入力",
                ),
            ],
            preferred_en(),
        ),
        rule(
            &format!("preset-{prefix}-editor"),
            &format!("{app_name} Editor"),
            app_id,
            130,
            vec![condition(
                AxAttribute::DomIdentifier,
                MatchOperator::Contains,
                "workbench.parts.editor",
            )],
            preferred_en(),
        ),
    ]
}

pub fn default_rules() -> Vec<FocusRule> {
    let mut rules = vec![rule(
        "preset-terminal",
        "Terminal",
        TERMINAL_BUNDLE_ID,
        0,
        Vec::new(),
        preferred_en(),
    )];

    let mut cursor = contextual_rules("cursor", "Cursor", CURSOR_BUNDLE_ID);
    cursor.push(rule(
        "preset-cursor-chat",
        "Cursor Chat",
        CURSOR_BUNDLE_ID,
        110,
        vec![condition(
            AxAttribute::DomIdentifier,
            MatchOperator::Contains,
            "aichat",
        )],
        preferred_ja(),
    ));
    cursor.push(rule(
        "preset-cursor-composer",
        "Cursor Composer",
        CURSOR_BUNDLE_ID,
        111,
        vec![condition(
            AxAttribute::DomClassList,
            MatchOperator::Contains,
            "composer-",
        )],
        preferred_ja(),
    ));
    cursor.push(rule(
        "preset-cursor-ai-input",
        "Cursor AI Input",
        CURSOR_BUNDLE_ID,
        112,
        vec![condition(
            AxAttribute::DomClassList,
            MatchOperator::Contains,
            "aislash-editor-input",
        )],
        preferred_ja(),
    ));
    rules.extend(cursor);

    let mut vscode = contextual_rules("vscode", "VS Code", VSCODE_BUNDLE_ID);
    vscode.push(rule(
        "preset-vscode-chat",
        "VS Code Chat",
        VSCODE_BUNDLE_ID,
        110,
        vec![condition(
            AxAttribute::DomIdentifier,
            MatchOperator::Contains,
            "workbench.panel.chat",
        )],
        preferred_ja(),
    ));
    rules.extend(vscode);
    rules.extend(readytype_lab_switch_rules());

    rules
}

/// ReadyType 自身の Lab 画面で、フィールドごとに固定言語へ切替する。
/// 言語割当は 01/03/05 → ja、02/04/06 → en（設定の preferred を参照）。
pub fn readytype_lab_switch_rules() -> Vec<FocusRule> {
    LAB_SWITCH_FIELD_IDS
        .iter()
        .enumerate()
        .map(|(index, field_id)| {
            let ordinal = index + 1;
            let target = if ordinal % 2 == 1 {
                preferred_ja()
            } else {
                preferred_en()
            };
            let language = if ordinal % 2 == 1 { "ja" } else { "en" };
            rule(
                &format!("preset-readytype-lab-switch-{ordinal:02}"),
                &format!("ReadyType Lab Switch {ordinal:02} ({language})"),
                READYTYPE_BUNDLE_ID,
                200 + u32::try_from(index).expect("lab field index fits u32"),
                // NSTextField フォーカス時は field editor が FOCUSED になることがあるため祖先も見る。
                vec![condition(
                    AxAttribute::Identifier,
                    MatchOperator::Equals,
                    field_id,
                )],
                target,
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
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
}
