use super::{strip_app_extension, MacOsAppDisplayNameResolver};
use crate::application::ports::app_display_name::AppDisplayNameResolver;

#[test]
fn unknown_bundle_id_is_none() {
    assert!(MacOsAppDisplayNameResolver
        .resolve("com.readytype.missing.app")
        .is_none());
}

#[test]
fn strips_app_extension_from_display_name() {
    assert_eq!(strip_app_extension("Google Chrome.app"), "Google Chrome");
    assert_eq!(strip_app_extension("Terminal.APP"), "Terminal");
    assert_eq!(strip_app_extension("Cursor"), "Cursor");
    assert_eq!(strip_app_extension(".app"), ".app");
}

#[test]
fn terminal_bundle_resolves_display_name() {
    let name = match MacOsAppDisplayNameResolver.resolve("com.apple.Terminal") {
        Some(value) => value,
        None => {
            eprintln!("skip: NSWorkspace could not resolve com.apple.Terminal");
            return;
        }
    };
    assert!(!name.is_empty());
    assert!(
        !name.to_ascii_lowercase().ends_with(".app"),
        "display name should not include .app: {name}"
    );
    // Localized; English is "Terminal".
    assert!(
        name.contains("Terminal") || name.contains("ターミナル"),
        "unexpected display name: {name}"
    );
}
