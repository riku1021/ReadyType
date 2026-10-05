//! Resolve input-source display names for a specific UI locale.
//!
//! `kTISPropertyLocalizedName` follows the process CFBundle language and often
//! stays English in Tauri. Menu-bar names live in each IME's
//! `InfoPlist.strings` / `InfoPlist.loctable` keyed by InputModeID.

use std::path::{Path, PathBuf};

use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSDictionary, NSString, NSURL};

/// Prefer IME-bundle localization for `locale`; fall back to TIS LocalizedName.
pub fn resolve_display_name(
    locale: &str,
    source_id: &str,
    bundle_id: Option<&str>,
    mode_id: Option<&str>,
    icon_path: Option<&Path>,
    fallback: &str,
) -> String {
    if locale.is_empty() {
        return fallback.to_owned();
    }
    let roots = resource_roots(bundle_id, icon_path);
    let keys = lookup_keys(source_id, mode_id);
    let localizations = localization_candidates(locale);
    for root in &roots {
        if let Some(name) = lookup_in_bundle_root(root, &localizations, &keys) {
            return name;
        }
    }
    fallback.to_owned()
}

fn lookup_keys(source_id: &str, mode_id: Option<&str>) -> Vec<String> {
    let mut keys = Vec::new();
    if let Some(mode_id) = mode_id.filter(|value| !value.is_empty()) {
        keys.push(mode_id.to_owned());
    }
    keys.push(source_id.to_owned());
    if let Some(suffix) = source_id.strip_prefix("com.apple.keylayout.") {
        keys.push(suffix.to_owned());
    }
    keys
}

fn localization_candidates(locale: &str) -> Vec<&'static str> {
    match locale {
        "ja" => vec!["ja", "Japanese", "ja-JP", "ja_JP"],
        "en" => vec!["en", "English", "en-US", "en_US"],
        "zh" => vec!["zh-Hans", "zh_CN", "zh-CN", "zh", "Chinese"],
        _ => vec![],
    }
}

fn resource_roots(bundle_id: Option<&str>, icon_path: Option<&Path>) -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(path) = icon_path.and_then(bundle_root_from_icon_path) {
        push_unique(&mut roots, path);
    }
    if let Some(bundle_id) = bundle_id {
        if let Some(path) = application_bundle_path(bundle_id) {
            push_unique(&mut roots, path);
        }
        if bundle_id.contains("keyboardlayout") {
            push_unique(
                &mut roots,
                PathBuf::from("/System/Library/Keyboard Layouts/AppleKeyboardLayouts.bundle"),
            );
        }
    }
    roots
}

fn bundle_root_from_icon_path(path: &Path) -> Option<PathBuf> {
    let mut current = Some(path);
    while let Some(candidate) = current {
        if let Some(name) = candidate.file_name().and_then(|value| value.to_str()) {
            if name.ends_with(".appex") || name.ends_with(".app") || name.ends_with(".bundle") {
                return Some(candidate.to_path_buf());
            }
        }
        current = candidate.parent();
    }
    None
}

fn application_bundle_path(bundle_id: &str) -> Option<PathBuf> {
    let workspace = NSWorkspace::sharedWorkspace();
    let identifier = NSString::from_str(bundle_id);
    let url = workspace.URLForApplicationWithBundleIdentifier(&identifier)?;
    let path = url.path()?.to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

fn lookup_in_bundle_root(root: &Path, localizations: &[&str], keys: &[String]) -> Option<String> {
    let resources = resources_dir(root);
    for localization in localizations {
        let strings_path = resources
            .join(format!("{localization}.lproj"))
            .join("InfoPlist.strings");
        if let Some(name) = lookup_in_strings_file(&strings_path, keys) {
            return Some(name);
        }
    }
    let loctable_path = resources.join("InfoPlist.loctable");
    lookup_in_loctable(&loctable_path, localizations, keys)
}

fn resources_dir(root: &Path) -> PathBuf {
    let with_contents = root.join("Contents").join("Resources");
    if with_contents.is_dir() {
        with_contents
    } else {
        root.join("Resources")
    }
}

fn lookup_in_strings_file(path: &Path, keys: &[String]) -> Option<String> {
    let dict = load_plist_dictionary(path)?;
    for key in keys {
        if let Some(value) = string_value(&dict, key) {
            return Some(value);
        }
    }
    None
}

fn lookup_in_loctable(path: &Path, localizations: &[&str], keys: &[String]) -> Option<String> {
    let root = load_plist_dictionary(path)?;
    for localization in localizations {
        let Some(lang_dict) = dictionary_value(&root, localization) else {
            continue;
        };
        for key in keys {
            if let Some(value) = string_value(&lang_dict, key) {
                return Some(value);
            }
        }
    }
    None
}

fn load_plist_dictionary(path: &Path) -> Option<objc2::rc::Retained<NSDictionary>> {
    if !path.is_file() {
        return None;
    }
    let path_string = NSString::from_str(&path.to_string_lossy());
    let url = NSURL::fileURLWithPath(&path_string);
    // SAFETY: URL points at a plist file; returns nil when unreadable.
    // dictionaryWithContentsOfURL is deprecated but still the lightest loader without a plist crate.
    #[allow(deprecated)]
    unsafe {
        NSDictionary::dictionaryWithContentsOfURL(&url)
    }
}

fn string_value(dict: &NSDictionary, key: &str) -> Option<String> {
    let key = NSString::from_str(key);
    let value = dict.objectForKey(&key)?;
    let as_string = value.downcast_ref::<NSString>()?;
    let text = as_string.to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

fn dictionary_value(dict: &NSDictionary, key: &str) -> Option<objc2::rc::Retained<NSDictionary>> {
    let key = NSString::from_str(key);
    let value = dict.objectForKey(&key)?;
    value.downcast::<NSDictionary>().ok()
}

fn push_unique(roots: &mut Vec<PathBuf>, path: PathBuf) {
    if !roots.iter().any(|existing| existing == &path) {
        roots.push(path);
    }
}

/// CFURL / NSURL path helper used by TIS icon property.
pub fn path_from_cfurl(url: *const std::ffi::c_void) -> Option<PathBuf> {
    if url.is_null() {
        return None;
    }
    // SAFETY: TIS IconImageURL values are CFURL / NSURL instances.
    let nsurl = unsafe { &*(url as *const NSURL) };
    let path = nsurl
        .absoluteURL()
        .and_then(|absolute| absolute.path())
        .or_else(|| nsurl.path())?
        .to_string();
    if path.is_empty() {
        None
    } else {
        Some(PathBuf::from(path))
    }
}

#[cfg(test)]
mod tests {
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
}
