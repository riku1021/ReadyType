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
