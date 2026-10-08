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
