//! Quick Set session: Level-1 hotkey popover data and apply logic helpers.

use std::collections::BTreeSet;

use crate::domain::input_source::{
    propose_conditions, propose_field_label, propose_rule_name, FocusRule, FocusSnapshot,
    LanguageTag, Platform, RuleCondition, RuleTarget,
};

use super::ports::input_source::AvailableInputSource;
use super::settings::{group_sources_by_language, pick_default_source_id, SwitcherSettings};

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

    let languages = build_quick_set_languages(rules, settings, sources);

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

/// Languages offered in Quick Set: OS available groups ∪ rule PreferredLanguage tags
/// ∪ fixed preferences, each resolved to an available input source when possible.
pub fn build_quick_set_languages(
    rules: &[FocusRule],
    settings: &SwitcherSettings,
    sources: &[AvailableInputSource],
) -> Vec<QuickSetLanguageOption> {
    let by_language = group_sources_by_language(sources);
    let mut tags = BTreeSet::new();

    for (tag, group) in &by_language {
        if group.iter().any(|source| source.available) {
            tags.insert(tag.clone());
        }
    }
    for rule in rules {
        if let RuleTarget::PreferredLanguage(tag) = rule.target() {
            tags.insert(tag.clone());
        }
    }
    for (tag, preferred) in &settings.language_preferences {
        if preferred.is_some() {
            tags.insert(tag.clone());
        }
    }

    tags.into_iter()
        .filter_map(|tag| resolve_language_option(&tag, settings, sources, &by_language))
        .collect()
}

fn resolve_language_option(
    tag: &LanguageTag,
    settings: &SwitcherSettings,
    sources: &[AvailableInputSource],
    by_language: &std::collections::BTreeMap<LanguageTag, Vec<&AvailableInputSource>>,
) -> Option<QuickSetLanguageOption> {
    if let Some(preferred) = settings.preferred(tag) {
        if let Some(source) = sources.iter().find(|source| &source.id == preferred) {
            if source.available {
                return Some(QuickSetLanguageOption {
                    language: tag.clone(),
                    preferred_source_id: preferred.as_str().to_owned(),
                    preferred_source_name: source.name.clone(),
                });
            }
        } else if sources.is_empty() {
            // Unit tests / offline: keep configured preference when OS list is absent.
            return Some(QuickSetLanguageOption {
                language: tag.clone(),
                preferred_source_id: preferred.as_str().to_owned(),
                preferred_source_name: preferred.as_str().to_owned(),
            });
        }
    }

    let group = by_language.get(tag)?;
    let source_id = pick_default_source_id(group)?;
    let name = sources
        .iter()
        .find(|source| source.id == source_id)
        .map(|source| source.name.clone())
        .unwrap_or_else(|| source_id.as_str().to_owned());
    Some(QuickSetLanguageOption {
        language: tag.clone(),
        preferred_source_id: source_id.as_str().to_owned(),
        preferred_source_name: name,
    })
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
    use crate::application::ports::input_source::AvailableInputSource;
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

    fn source(id: &str, name: &str, language: &str) -> AvailableInputSource {
        AvailableInputSource {
            id: InputSourceId::parse(id).expect("id"),
            name: name.to_owned(),
            language_code: Some(language.to_owned()),
            available: true,
        }
    }

    fn rule_preferred(id: &str, tag: LanguageTag) -> FocusRule {
        FocusRule::new(
            id,
            id,
            true,
            Platform::MacOs,
            "com.example.app",
            100,
            Vec::new(),
            RuleTarget::PreferredLanguage(tag),
        )
        .expect("rule")
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

    #[test]
    fn build_quick_set_languages_unions_os_rules_and_preferences() {
        let en = source("com.apple.keylayout.ABC", "ABC", "en");
        let ja = source("com.apple.inputmethod.Kotoeri.Japanese", "Hiragana", "ja");
        let ko = source("com.apple.inputmethod.Korean", "2-Set Korean", "ko");
        let sources = vec![en.clone(), ja.clone(), ko.clone()];

        let cases = [
            (
                "os groups only",
                Vec::<FocusRule>::new(),
                SwitcherSettings {
                    language_preferences: Default::default(),
                    rules: Vec::new(),
                    ..SwitcherSettings::defaults()
                },
                sources.clone(),
                vec!["en", "ja", "ko"],
            ),
            (
                "prefs-only language kept offline",
                Vec::new(),
                {
                    let mut settings = SwitcherSettings {
                        rules: Vec::new(),
                        ..SwitcherSettings::defaults()
                    };
                    settings
                        .language_preferences
                        .insert(LanguageTag::ja(), Some(ja.id.clone()));
                    settings
                },
                Vec::new(),
                vec!["ja"],
            ),
            (
                "rule tag resolves via os default",
                vec![rule_preferred("r-ja", LanguageTag::ja())],
                SwitcherSettings {
                    language_preferences: Default::default(),
                    rules: Vec::new(),
                    ..SwitcherSettings::defaults()
                },
                vec![en.clone(), ja.clone()],
                vec!["en", "ja"],
            ),
            (
                "stale preference falls back to os group",
                Vec::new(),
                {
                    let mut settings = SwitcherSettings {
                        rules: Vec::new(),
                        ..SwitcherSettings::defaults()
                    };
                    settings.language_preferences.insert(
                        LanguageTag::ja(),
                        Some(InputSourceId::parse("missing.ja").expect("id")),
                    );
                    settings
                },
                vec![ja.clone()],
                vec!["ja"],
            ),
            (
                "unavailable rule language omitted",
                vec![rule_preferred(
                    "r-zh",
                    LanguageTag::parse("zh").expect("zh"),
                )],
                SwitcherSettings {
                    language_preferences: Default::default(),
                    rules: Vec::new(),
                    ..SwitcherSettings::defaults()
                },
                vec![en.clone()],
                vec!["en"],
            ),
        ];

        for (name, rules, settings, case_sources, expected_tags) in cases {
            let options = build_quick_set_languages(&rules, &settings, &case_sources);
            let tags: Vec<String> = options
                .iter()
                .map(|option| option.language.as_str().to_owned())
                .collect();
            assert_eq!(tags, expected_tags, "{name}");
        }

        let mut settings = SwitcherSettings {
            rules: Vec::new(),
            ..SwitcherSettings::defaults()
        };
        settings
            .language_preferences
            .insert(LanguageTag::en(), Some(en.id.clone()));
        let options = build_quick_set_languages(&[], &settings, &sources);
        let en_option = options
            .iter()
            .find(|option| option.language == LanguageTag::en())
            .expect("en");
        assert_eq!(en_option.preferred_source_id, en.id.as_str());
        assert_eq!(en_option.preferred_source_name, "ABC");
    }
}
