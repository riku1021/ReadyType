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
