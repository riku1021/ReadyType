use super::{
    AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot, InputSourceId, MatchOperator,
    Platform, PlatformAppId, RuleCondition, RuleSet, RuleTarget, SearchScope,
};

const CURSOR_BUNDLE_ID: &str = "com.todesktop.230313mzl4w4u92";

fn app(platform: Platform, app_id: &str, pid: i32) -> PlatformAppId {
    PlatformAppId::new(platform, app_id, pid).expect("valid app")
}

fn element(role: &str, dom_identifier: &str) -> AxElementSnapshot {
    AxElementSnapshot::new()
        .with(AxAttribute::Role, role)
        .with(AxAttribute::DomIdentifier, dom_identifier)
}

fn condition(
    attribute: AxAttribute,
    operator: MatchOperator,
    scope: SearchScope,
    value: &str,
) -> RuleCondition {
    RuleCondition::new(attribute, operator, scope, value).expect("valid condition")
}

fn rule(
    id: &str,
    enabled: bool,
    platform: Platform,
    app_id: &str,
    priority: u32,
    conditions: Vec<RuleCondition>,
) -> FocusRule {
    FocusRule::new(
        id,
        id,
        enabled,
        platform,
        app_id,
        priority,
        conditions,
        RuleTarget::PreferredEnglish,
    )
    .expect("valid rule")
}

fn cursor_snapshot(generation: u64, elements: Vec<AxElementSnapshot>) -> FocusSnapshot {
    FocusSnapshot::new(
        app(Platform::MacOs, CURSOR_BUNDLE_ID, 42),
        generation,
        elements,
    )
}

#[test]
fn all_conditions_may_match_different_ancestors() {
    let rule = rule(
        "cursor-chat",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        0,
        vec![
            condition(
                AxAttribute::Role,
                MatchOperator::Equals,
                SearchScope::FocusedOnly,
                "AXTextArea",
            ),
            condition(
                AxAttribute::DomIdentifier,
                MatchOperator::Contains,
                SearchScope::FocusedAndAncestors,
                "aichat",
            ),
        ],
    );
    let snapshot = cursor_snapshot(
        7,
        vec![
            element("AXTextArea", ""),
            element("AXGroup", "workbench.panel.aichat"),
        ],
    );

    assert!(rule.matches(&snapshot));
}

#[test]
fn comparisons_ignore_case_for_equals_and_contains() {
    let snapshot = cursor_snapshot(
        1,
        vec![element("AXTEXTAREA", "WORKBENCH.PANEL.AICHAT.INPUT")],
    );
    let conditions = [
        condition(
            AxAttribute::Role,
            MatchOperator::Equals,
            SearchScope::FocusedOnly,
            "axtextarea",
        ),
        condition(
            AxAttribute::DomIdentifier,
            MatchOperator::Contains,
            SearchScope::FocusedAndAncestors,
            "aichat",
        ),
    ];

    assert!(conditions
        .iter()
        .all(|item| item.matches(snapshot.elements())));
}

#[test]
fn focused_only_does_not_match_an_ancestor() {
    let condition = condition(
        AxAttribute::Role,
        MatchOperator::Equals,
        SearchScope::FocusedOnly,
        "AXGroup",
    );
    let snapshot = cursor_snapshot(1, vec![element("AXTextArea", ""), element("AXGroup", "")]);

    assert!(!condition.matches(snapshot.elements()));
}

#[test]
fn rule_without_conditions_matches_its_application() {
    let terminal = rule(
        "terminal",
        true,
        Platform::MacOs,
        "com.apple.Terminal",
        0,
        Vec::new(),
    );
    let snapshot = FocusSnapshot::new(app(Platform::MacOs, "com.apple.Terminal", 9), 1, Vec::new());

    assert!(terminal.matches(&snapshot));
}

#[test]
fn rule_set_skips_disabled_rules_and_returns_lowest_priority() {
    let disabled = rule(
        "disabled",
        false,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        0,
        Vec::new(),
    );
    let later = rule(
        "later",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        20,
        Vec::new(),
    );
    let first = rule(
        "first",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        10,
        Vec::new(),
    );
    let rules = RuleSet::new(vec![later, disabled, first]).expect("valid rules");

    assert_eq!(
        rules
            .first_match(&cursor_snapshot(1, Vec::new()))
            .map(FocusRule::id),
        Some("first")
    );
}

#[test]
fn rejects_empty_condition_value_and_duplicate_rule_ids() {
    assert!(RuleCondition::new(
        AxAttribute::Description,
        MatchOperator::Contains,
        SearchScope::FocusedAndAncestors,
        "   ",
    )
    .is_err());

    let first = rule(
        "same",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        0,
        Vec::new(),
    );
    let second = rule(
        "same",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        1,
        Vec::new(),
    );
    assert!(RuleSet::new(vec![first, second]).is_err());
}

#[test]
fn rejects_blank_input_source_and_normalizes_surrounding_whitespace() {
    assert!(InputSourceId::parse("   ").is_err());
    assert_eq!(
        InputSourceId::parse("  com.apple.keylayout.ABC  ")
            .expect("valid input source")
            .as_str(),
        "com.apple.keylayout.ABC"
    );
}

#[test]
fn wrong_platform_or_application_does_not_match() {
    let cursor = rule(
        "cursor",
        true,
        Platform::MacOs,
        CURSOR_BUNDLE_ID,
        0,
        Vec::new(),
    );
    let windows_snapshot =
        FocusSnapshot::new(app(Platform::Windows, CURSOR_BUNDLE_ID, 42), 1, Vec::new());
    let other_app_snapshot =
        FocusSnapshot::new(app(Platform::MacOs, "com.example.Other", 42), 1, Vec::new());

    assert!(!cursor.matches(&windows_snapshot));
    assert!(!cursor.matches(&other_app_snapshot));
}
