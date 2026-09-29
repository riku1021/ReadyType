//! Tray menu strings and Rust-side supported UI locale packs.
//!
//! Source of truth for which packs exist on the native side:
//! `rust-native/i18n/tray.json` keys (not the FE registry).

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

const TRAY_JSON: &str = include_str!("../../i18n/tray.json");

/// Stored preference when unset / invalid.
pub const DEFAULT_UI_LOCALE_PREFERENCE: &str = "system";
/// Fallback when OS language is unsupported. Must exist as a key in `tray.json`.
pub const DEFAULT_RESOLVED_UI_LOCALE: &str = "ja";

#[derive(Debug, Clone, Deserialize)]
pub struct TrayStrings {
    pub status_snoozed: String,
    pub status_watching: String,
    pub status_watching_app: String,
    pub status_watching_app_rule: String,
    pub status_off: String,
    pub toggle_off: String,
    pub toggle_on: String,
    pub snooze_5m: String,
    pub snooze_15m: String,
    pub snooze_1h: String,
    pub snooze_indefinite: String,
    pub resume: String,
    pub open_app: String,
    pub open_settings: String,
    pub quit: String,
}

#[derive(Debug)]
struct TrayCatalog {
    strings: HashMap<String, TrayStrings>,
    /// Stable order for `UiLocaleCache` encoding (sorted by code).
    codes: Vec<String>,
}

static TRAY_CATALOG: LazyLock<TrayCatalog> = LazyLock::new(|| {
    let strings: HashMap<String, TrayStrings> =
        serde_json::from_str(TRAY_JSON).expect("rust-native/i18n/tray.json must be valid");
    assert!(
        !strings.is_empty(),
        "tray.json must define at least one locale pack"
    );
    assert!(
        strings.contains_key(DEFAULT_RESOLVED_UI_LOCALE),
        "tray.json must include default resolved locale '{DEFAULT_RESOLVED_UI_LOCALE}'"
    );
    let mut codes: Vec<String> = strings.keys().cloned().collect();
    codes.sort();
    TrayCatalog { strings, codes }
});

fn catalog() -> &'static TrayCatalog {
    &TRAY_CATALOG
}

pub fn tray_strings(locale: &str) -> &'static TrayStrings {
    let catalog = catalog();
    catalog
        .strings
        .get(locale)
        .or_else(|| catalog.strings.get(DEFAULT_RESOLVED_UI_LOCALE))
        .expect("default resolved tray pack must exist")
}

pub fn supported_codes() -> &'static [String] {
    &catalog().codes
}

pub fn is_supported_ui_locale(code: &str) -> bool {
    catalog().strings.contains_key(code)
}

pub fn index_of(code: &str) -> Option<usize> {
    catalog().codes.iter().position(|entry| entry == code)
}

pub fn code_at(index: usize) -> Option<&'static str> {
    catalog().codes.get(index).map(String::as_str)
}

pub fn default_resolved_index() -> usize {
    index_of(DEFAULT_RESOLVED_UI_LOCALE).expect("default resolved present in tray.json")
}

/// Map an OS / BCP-47 tag to a tray pack via primary subtag.
pub fn map_language_tag(tag: &str) -> Option<&'static str> {
    let primary = tag
        .split(['-', '_'])
        .next()
        .unwrap_or(tag)
        .to_ascii_lowercase();
    catalog()
        .codes
        .iter()
        .find(|code| code.as_str() == primary)
        .map(String::as_str)
}

#[cfg(test)]
mod tests {
    use super::{
        is_supported_ui_locale, map_language_tag, supported_codes, tray_strings,
        DEFAULT_RESOLVED_UI_LOCALE, DEFAULT_UI_LOCALE_PREFERENCE,
    };

    #[test]
    fn loads_packs_and_falls_back_unknown_to_default() {
        assert_eq!(tray_strings("zh").quit, "退出");
        assert_eq!(tray_strings("en").quit, "Quit");
        assert_eq!(tray_strings("nope").quit, "終了");
    }

    #[test]
    fn supported_codes_come_from_tray_keys() {
        assert_eq!(DEFAULT_UI_LOCALE_PREFERENCE, "system");
        assert_eq!(DEFAULT_RESOLVED_UI_LOCALE, "ja");
        assert!(is_supported_ui_locale("ja"));
        assert!(is_supported_ui_locale("en"));
        assert!(is_supported_ui_locale("zh"));
        assert!(!is_supported_ui_locale("fr"));
        assert!(
            supported_codes().windows(2).all(|pair| pair[0] <= pair[1]),
            "supported codes must stay sorted for UiLocaleCache indices"
        );
    }

    #[test]
    fn map_language_tag_uses_primary_subtag() {
        assert_eq!(map_language_tag("ja-JP"), Some("ja"));
        assert_eq!(map_language_tag("en_US"), Some("en"));
        assert_eq!(map_language_tag("zh-CN"), Some("zh"));
        assert_eq!(map_language_tag("zh_TW"), Some("zh"));
        assert_eq!(map_language_tag("fr-FR"), None);
    }
}
