use std::collections::BTreeMap;

use crate::application::tray_strings::{self, map_language_tag};
use crate::domain::{
    errors::DomainError,
    input_source::{default_rules, FocusRule, InputSourceId, LanguageTag, RuleSet},
};

pub use crate::application::tray_strings::{
    DEFAULT_RESOLVED_UI_LOCALE, DEFAULT_UI_LOCALE_PREFERENCE,
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

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
    /// UI language preference (`system` | tray.json pack code).
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

    /// Normalize a stored preference to `system` or a tray-supported UI pack.
    pub fn normalize_ui_locale(locale: &str) -> String {
        if locale == "system" || tray_strings::is_supported_ui_locale(locale) {
            return locale.to_owned();
        }
        DEFAULT_UI_LOCALE_PREFERENCE.to_owned()
    }

    /// Resolve preference to a concrete UI language pack.
    pub fn resolve_ui_locale(preference: &str) -> String {
        let normalized = Self::normalize_ui_locale(preference);
        if tray_strings::is_supported_ui_locale(&normalized) {
            return normalized;
        }
        resolve_os_ui_locale()
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
    let by_language = group_sources_by_language(sources);

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

/// OS 一覧と突き合わせ、消失・言語不一致の preference を差し替える。
pub fn reconcile_language_preferences(
    preferences: &BTreeMap<LanguageTag, Option<InputSourceId>>,
    sources: &[crate::application::ports::input_source::AvailableInputSource],
) -> BTreeMap<LanguageTag, Option<InputSourceId>> {
    let by_language = group_sources_by_language(sources);
    let mut result = BTreeMap::new();
    for (tag, preferred) in preferences {
        let still_valid = preferred.as_ref().is_some_and(|id| {
            sources.iter().any(|source| {
                source.available
                    && &source.id == id
                    && source
                        .language_code
                        .as_deref()
                        .and_then(|code| LanguageTag::parse(code).ok())
                        .as_ref()
                        == Some(tag)
            })
        });
        if still_valid {
            result.insert(tag.clone(), preferred.clone());
        } else {
            let replacement = by_language
                .get(tag)
                .and_then(|group| pick_default_source_id(group));
            result.insert(tag.clone(), replacement);
        }
    }
    result
}

fn group_sources_by_language(
    sources: &[crate::application::ports::input_source::AvailableInputSource],
) -> BTreeMap<LanguageTag, Vec<&crate::application::ports::input_source::AvailableInputSource>> {
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
    by_language
}

fn pick_default_source_id(
    group: &[&crate::application::ports::input_source::AvailableInputSource],
) -> Option<InputSourceId> {
    let available = group
        .iter()
        .copied()
        .filter(|source| source.available)
        .collect::<Vec<_>>();
    available
        .iter()
        .find(|source| !crate::domain::input_source::is_ime_latin_mode(source.id.as_str()))
        .or_else(|| available.first())
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

#[cfg(test)]
mod tests {
    use super::{
        reconcile_language_preferences, seed_language_preferences_from_sources, SwitcherSettings,
        CURRENT_SCHEMA_VERSION, DEFAULT_UI_LOCALE_PREFERENCE,
    };
    use crate::{
        application::ports::input_source::AvailableInputSource,
        domain::input_source::{
            canonicalize_language_codes_by_family, is_ime_latin_mode, InputSourceId, LanguageTag,
        },
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
    fn family_canonicalization_maps_ime_latin_modes_without_language_whitelist() {
        assert!(is_ime_latin_mode("com.google.inputmethod.Japanese.Roman"));
        assert!(is_ime_latin_mode("com.apple.inputmethod.Korean.Roman"));
        assert!(!is_ime_latin_mode(
            "com.google.inputmethod.Japanese.FullWidthRoman"
        ));
        assert!(!is_ime_latin_mode("com.apple.keylayout.ABC"));

        let mut entries = vec![
            ("com.apple.keylayout.ABC".to_owned(), Some("en".to_owned())),
            (
                "com.google.inputmethod.Japanese.base".to_owned(),
                Some("ja".to_owned()),
            ),
            (
                "com.google.inputmethod.Japanese.Roman".to_owned(),
                Some("en".to_owned()),
            ),
            (
                "com.apple.inputmethod.Korean.2SetKorean".to_owned(),
                Some("ko".to_owned()),
            ),
            (
                "com.apple.inputmethod.Korean.Roman".to_owned(),
                Some("en".to_owned()),
            ),
            (
                "com.apple.keylayout.ABC-AZERTY".to_owned(),
                Some("fr".to_owned()),
            ),
        ];
        canonicalize_language_codes_by_family(&mut entries);

        assert_eq!(entries[0].1.as_deref(), Some("en")); // real English layout stays
        assert_eq!(entries[1].1.as_deref(), Some("ja"));
        assert_eq!(entries[2].1.as_deref(), Some("ja")); // Japanese IME latin → ja
        assert_eq!(entries[3].1.as_deref(), Some("ko"));
        assert_eq!(entries[4].1.as_deref(), Some("ko")); // Korean IME latin → ko
        assert_eq!(entries[5].1.as_deref(), Some("fr"));
    }

    #[test]
    fn reconcile_replaces_unavailable_or_mismatched_preferences() {
        let abc = InputSourceId::parse("com.apple.keylayout.ABC").expect("abc");
        let roman = InputSourceId::parse("com.google.inputmethod.Japanese.Roman").expect("roman");
        let base = InputSourceId::parse("com.google.inputmethod.Japanese.base").expect("base");
        let mut preferences = std::collections::BTreeMap::new();
        preferences.insert(LanguageTag::en(), Some(roman.clone()));
        preferences.insert(LanguageTag::ja(), Some(base.clone()));

        let sources = vec![
            AvailableInputSource {
                id: abc.clone(),
                name: "ABC".to_owned(),
                language_code: Some("en".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: base.clone(),
                name: "Hiragana".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
            // Roman is available but classified as ja (canonical), so invalid for en.
            AvailableInputSource {
                id: roman,
                name: "Alphanumeric".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
        ];

        let reconciled = reconcile_language_preferences(&preferences, &sources);
        assert_eq!(
            reconciled.get(&LanguageTag::en()).cloned().flatten(),
            Some(abc)
        );
        assert_eq!(
            reconciled.get(&LanguageTag::ja()).cloned().flatten(),
            Some(base)
        );
    }

    #[test]
    fn pick_default_skips_ime_latin_modes() {
        let roman = InputSourceId::parse("com.google.inputmethod.Japanese.Roman").expect("roman");
        let base = InputSourceId::parse("com.google.inputmethod.Japanese.base").expect("base");
        let sources = vec![
            AvailableInputSource {
                id: roman,
                name: "Alphanumeric".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
            AvailableInputSource {
                id: base.clone(),
                name: "Hiragana".to_owned(),
                language_code: Some("ja".to_owned()),
                available: true,
            },
        ];
        let seeded = seed_language_preferences_from_sources(&sources, None);
        assert_eq!(
            seeded.get(&LanguageTag::ja()).cloned().flatten(),
            Some(base)
        );
    }

    #[test]
    fn normalize_ui_locale_accepts_system_and_tray_packs() {
        assert_eq!(SwitcherSettings::normalize_ui_locale("en"), "en");
        assert_eq!(SwitcherSettings::normalize_ui_locale("ja"), "ja");
        assert_eq!(SwitcherSettings::normalize_ui_locale("zh"), "zh");
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
        assert_eq!(SwitcherSettings::resolve_ui_locale("zh"), "zh");
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
