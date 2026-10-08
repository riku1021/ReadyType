use super::{
    bundle_root_from_icon_path, localization_candidates, lookup_in_bundle_root, lookup_keys,
    resolve_display_name,
};
use std::path::PathBuf;

#[test]
fn lookup_keys_prefer_mode_id_then_layout_suffix() {
    assert_eq!(
        lookup_keys(
            "com.google.inputmethod.Japanese.base",
            Some("com.apple.inputmethod.Japanese")
        ),
        vec![
            "com.apple.inputmethod.Japanese".to_owned(),
            "com.google.inputmethod.Japanese.base".to_owned(),
        ]
    );
    assert_eq!(
        lookup_keys("com.apple.keylayout.ABC", None),
        vec!["com.apple.keylayout.ABC".to_owned(), "ABC".to_owned()]
    );
}

#[test]
fn localization_candidates_cover_legacy_japanese_lproj() {
    assert!(localization_candidates("ja").contains(&"Japanese"));
    assert!(localization_candidates("ja").contains(&"ja"));
}

#[test]
fn bundle_root_walks_up_to_appex_or_app() {
    let path = PathBuf::from(
        "/Library/Input Methods/GoogleJapaneseInput.app/Contents/Resources/hiragana.tiff",
    );
    assert_eq!(
        bundle_root_from_icon_path(&path),
        Some(PathBuf::from(
            "/Library/Input Methods/GoogleJapaneseInput.app"
        ))
    );
}

fn fixture_root(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/input_source_names")
        .join(name)
}

#[test]
fn google_japanese_lproj_resolves_hiragana_for_ja() {
    let root = fixture_root("google_ime");
    let name = lookup_in_bundle_root(
        &root,
        &["ja", "Japanese"],
        &["com.apple.inputmethod.Japanese".to_owned()],
    );
    assert_eq!(name.as_deref(), Some("ひらがな (Google)"));
    let roman = lookup_in_bundle_root(
        &root,
        &["ja", "Japanese"],
        &["com.apple.inputmethod.Roman".to_owned()],
    );
    assert_eq!(roman.as_deref(), Some("英数 (Google)"));
}

#[test]
fn apple_japanese_loctable_resolves_hiragana_for_ja() {
    let root = fixture_root("apple_japanese_appex");
    let name = lookup_in_bundle_root(
        &root,
        &["ja"],
        &["com.apple.inputmethod.Japanese".to_owned()],
    );
    assert_eq!(name.as_deref(), Some("ひらがな"));
}

#[test]
fn resolve_uses_fallback_when_locale_empty() {
    assert_eq!(
        resolve_display_name("", "id", None, None, None, "Fallback"),
        "Fallback"
    );
}
