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
pub const READYTYPE_BUNDLE_ID: &str = "com.readytype";

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
mod tests;
