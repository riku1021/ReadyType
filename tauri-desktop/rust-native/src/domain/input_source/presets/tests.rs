use super::{readytype_lab_switch_rules, LAB_SWITCH_FIELD_IDS};
use crate::domain::input_source::{
    AxAttribute, AxElementSnapshot, FocusSnapshot, LanguageTag, Platform, PlatformAppId, RuleSet,
    RuleTarget,
};

fn element(values: &[(AxAttribute, &str)]) -> AxElementSnapshot {
    values
        .iter()
        .fold(AxElementSnapshot::new(), |element, (attribute, value)| {
            element.with(*attribute, *value)
        })
}

#[test]
fn lab_switch_rule_ids_are_stable_and_unique() {
    let rules = readytype_lab_switch_rules();
    let ids = rules
        .iter()
        .map(|rule| rule.id().to_owned())
        .collect::<Vec<_>>();

    assert_eq!(ids.len(), LAB_SWITCH_FIELD_IDS.len());
    assert!(ids.iter().any(|id| id == "preset-readytype-lab-switch-01"));
    assert!(ids.iter().any(|id| id == "preset-readytype-lab-switch-06"));
    assert_eq!(
        RuleSet::new(rules).expect("unique lab rules").rules().len(),
        ids.len()
    );
}

#[test]
fn readytype_lab_switch_fields_match_fixed_languages() {
    let app = PlatformAppId::new(Platform::MacOs, "com.readytype", 1).expect("valid app");
    let rules = RuleSet::new(readytype_lab_switch_rules()).expect("valid lab rules");

    let cases = [
        (
            "readytype-lab-switch-01",
            "preset-readytype-lab-switch-01",
            LanguageTag::ja(),
        ),
        (
            "readytype-lab-switch-02",
            "preset-readytype-lab-switch-02",
            LanguageTag::en(),
        ),
        (
            "readytype-lab-switch-03",
            "preset-readytype-lab-switch-03",
            LanguageTag::ja(),
        ),
        (
            "readytype-lab-switch-04",
            "preset-readytype-lab-switch-04",
            LanguageTag::en(),
        ),
        (
            "readytype-lab-switch-05",
            "preset-readytype-lab-switch-05",
            LanguageTag::ja(),
        ),
        (
            "readytype-lab-switch-06",
            "preset-readytype-lab-switch-06",
            LanguageTag::en(),
        ),
    ];
    for (identifier, expected_rule_id, expected_language) in cases {
        let snapshot = FocusSnapshot::new(
            app.clone(),
            1,
            vec![element(&[(AxAttribute::Identifier, identifier)])],
        );
        let matched = rules.first_match(&snapshot).expect("lab field matches");
        assert_eq!(matched.id(), expected_rule_id);
        match matched.target() {
            RuleTarget::PreferredLanguage(tag) => assert_eq!(tag, &expected_language),
            other => panic!("expected preferred language, got {other:?}"),
        }

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
