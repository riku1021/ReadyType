use std::collections::BTreeMap;

use crate::domain::{
    errors::DomainError,
    input_source::{default_rules, FocusRule, InputSourceId, LanguageTag, RuleSet},
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;
/// Stored preference when unset / invalid.
pub const DEFAULT_UI_LOCALE_PREFERENCE: &str = "system";
/// Fallback when OS language is unsupported or unavailable.
pub const DEFAULT_RESOLVED_UI_LOCALE: &str = "ja";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitcherSettings {
    pub schema_version: u32,
    pub enabled: bool,
    /// Per-language preferred input source. `None` value = unset.
    /// Empty map = not yet seeded from OS (see `seed_language_preferences_from_sources`).
    pub language_preferences: BTreeMap<LanguageTag, Option<InputSourceId>>,
    pub rules: Vec<FocusRule>,
    pub debug_logging: bool,
    pub snoozed_until_unix_ms: Option<u64>,
    /// UI language preference (`system` | `ja` | `en`).
    pub ui_locale: String,
}

impl SwitcherSettings {
    pub fn defaults() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled: true,
            language_preferences: BTreeMap::new(),
            rules: default_rules(),
            debug_logging: false,
            snoozed_until_unix_ms: None,
            ui_locale: DEFAULT_UI_LOCALE_PREFERENCE.to_owned(),
        }
    }

    pub fn preferred(&self, tag: &LanguageTag) -> Option<&InputSourceId> {
        self.language_preferences
            .get(tag)
            .and_then(|value| value.as_ref())
    }

    /// Normalize a stored preference to `system` | `ja` | `en`.
    pub fn normalize_ui_locale(locale: &str) -> String {
        match locale {
            "en" => "en".to_owned(),
            "ja" => "ja".to_owned(),
            "system" => "system".to_owned(),
            _ => DEFAULT_UI_LOCALE_PREFERENCE.to_owned(),
        }
    }

    /// Resolve preference to a concrete UI language (`ja` | `en`).
    pub fn resolve_ui_locale(preference: &str) -> String {
        match Self::normalize_ui_locale(preference).as_str() {
            "en" => "en".to_owned(),
            "ja" => "ja".to_owned(),
            _ => resolve_os_ui_locale(),
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(DomainError::InvalidInput(format!(
                "unsupported settings schema version: {}",
                self.schema_version
            )));
        }
        RuleSet::new(self.rules.clone())?;
        Ok(())
    }
}

/// OS 駆動の初期シード。言語ホワイトリストは持たない。
/// 現在の入力ソースの言語を 1 スロット。無ければ一覧の先頭言語グループを 1 つ。
pub fn seed_language_preferences_from_sources(
    sources: &[crate::application::ports::input_source::AvailableInputSource],
    current: Option<&InputSourceId>,
) -> BTreeMap<LanguageTag, Option<InputSourceId>> {
    let mut by_language: BTreeMap<
        LanguageTag,
        Vec<&crate::application::ports::input_source::AvailableInputSource>,
    > = BTreeMap::new();
    for source in sources {
        let Some(code) = source.language_code.as_deref() else {
            continue;
        };
        let Ok(tag) = LanguageTag::parse(code) else {
            continue;
        };
        by_language.entry(tag).or_default().push(source);
    }

    let mut preferences = BTreeMap::new();
    if let Some(current_id) = current {
        if let Some(current_source) = sources.iter().find(|source| &source.id == current_id) {
            if let Some(code) = current_source.language_code.as_deref() {
                if let Ok(tag) = LanguageTag::parse(code) {
                    if let Some(group) = by_language.get(&tag) {
                        preferences.insert(tag, pick_default_source_id(group));
                        return preferences;
                    }
                }
            }
        }
    }

    if let Some((tag, group)) = by_language.into_iter().next() {
        preferences.insert(tag, pick_default_source_id(&group));
    }
    preferences
}

fn pick_default_source_id(
    group: &[&crate::application::ports::input_source::AvailableInputSource],
) -> Option<InputSourceId> {
    group
        .iter()
        .find(|source| source.available)
        .or_else(|| group.first())
        .map(|source| source.id.clone())
}

fn resolve_os_ui_locale() -> String {
    for tag in sys_locale::get_locales() {
        if let Some(resolved) = map_language_tag(&tag) {
            return resolved.to_owned();
        }
    }
    if let Some(tag) = sys_locale::get_locale() {
        if let Some(resolved) = map_language_tag(&tag) {
            return resolved.to_owned();
        }
    }
    DEFAULT_RESOLVED_UI_LOCALE.to_owned()
}

fn map_language_tag(tag: &str) -> Option<&'static str> {
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or(tag)
        .to_ascii_lowercase();
    match primary.as_str() {
        "en" => Some("en"),
        "ja" => Some("ja"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        map_language_tag, seed_language_preferences_from_sources, SwitcherSettings,
        CURRENT_SCHEMA_VERSION, DEFAULT_RESOLVED_UI_LOCALE, DEFAULT_UI_LOCALE_PREFERENCE,
    };
    use crate::{
        application::ports::input_source::AvailableInputSource,
        domain::input_source::{InputSourceId, LanguageTag},
    };

    #[test]
    fn defaults_include_terminal_cursor_and_vscode() {
        let settings = SwitcherSettings::defaults();

        assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.apple.Terminal"));
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.todesktop.230313mzl4w4u92"));
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.microsoft.VSCode"));
    }

    #[test]
    fn defaults_start_with_empty_language_preferences() {
        let settings = SwitcherSettings::defaults();

        assert!(settings.enabled);
        assert!(settings.language_preferences.is_empty());
        assert!(!settings.debug_logging);
        assert_eq!(settings.snoozed_until_unix_ms, None);
        assert_eq!(settings.ui_locale, DEFAULT_UI_LOCALE_PREFERENCE);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn seed_uses_current_source_language_only() {
        let en = InputSourceId::parse("en.abc").expect("en");
        let ja = InputSourceId::parse("ja.id").expect("ja");
        let ko = InputSourceId::parse("ko.id").expect("ko");
        let sources = vec![
            AvailableInputSource {
                id: en,
                name: "ABC".to_owned(),
                language_code: Some("en".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: ja,
                name: "Japanese".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: ko.clone(),
                name: "Korean".to_owned(),
                language_code: Some("ko".to_owned()),
                available: true,
            },
        ];

        let seeded = seed_language_preferences_from_sources(&sources, Some(&ko));
        assert_eq!(seeded.len(), 1);
        assert_eq!(
            seeded
                .get(&LanguageTag::parse("ko").expect("ko"))
                .cloned()
                .flatten(),
            Some(ko)
        );
    }

    #[test]
    fn seed_falls_back_to_first_language_group() {
        let en = InputSourceId::parse("en.abc").expect("en");
        let sources = vec![AvailableInputSource {
            id: en.clone(),
            name: "ABC".to_owned(),
            language_code: Some("en".to_owned()),
            available: true,
        }];
        let seeded = seed_language_preferences_from_sources(&sources, None);
        assert_eq!(seeded.get(&LanguageTag::en()).cloned().flatten(), Some(en));
    }

    #[test]
    fn normalize_ui_locale_accepts_system_ja_en() {
        assert_eq!(SwitcherSettings::normalize_ui_locale("en"), "en");
        assert_eq!(SwitcherSettings::normalize_ui_locale("ja"), "ja");
        assert_eq!(SwitcherSettings::normalize_ui_locale("system"), "system");
        assert_eq!(
            SwitcherSettings::normalize_ui_locale("fr"),
            DEFAULT_UI_LOCALE_PREFERENCE
        );
    }

    #[test]
    fn resolve_explicit_locales_without_os() {
        assert_eq!(SwitcherSettings::resolve_ui_locale("en"), "en");
        assert_eq!(SwitcherSettings::resolve_ui_locale("ja"), "ja");
    }

    #[test]
    fn map_language_tag_extracts_primary_subtag() {
        assert_eq!(map_language_tag("ja-JP"), Some("ja"));
        assert_eq!(map_language_tag("en_US"), Some("en"));
        assert_eq!(map_language_tag("fr-FR"), None);
        assert_eq!(
            SwitcherSettings::resolve_ui_locale("system"),
            SwitcherSettings::resolve_ui_locale("system")
        );
        let _ = DEFAULT_RESOLVED_UI_LOCALE;
    }

    #[test]
    fn rejects_a_schema_version_newer_than_supported() {
        let mut settings = SwitcherSettings::defaults();
        settings.schema_version = CURRENT_SCHEMA_VERSION + 1;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_rule_ids() {
        let mut settings = SwitcherSettings::defaults();
        settings.rules.push(settings.rules[0].clone());

        assert!(settings.validate().is_err());
    }

    #[test]
    fn language_tag_normalizes_primary_subtag() {
        assert_eq!(LanguageTag::parse("en-US").expect("parse").as_str(), "en");
        assert_eq!(LanguageTag::parse("JA_jp").expect("parse").as_str(), "ja");
        assert!(LanguageTag::parse("  ").is_err());
    }
}
