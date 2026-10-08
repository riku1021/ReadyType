use super::MacOsAppIconSource;
use crate::infrastructure::platform::app_icon_cache::AppIconSource;

#[test]
fn unknown_bundle_id_is_none() {
    assert!(MacOsAppIconSource
        .probe_mtime("com.readytype.missing.app")
        .is_none());
    assert!(MacOsAppIconSource
        .fetch("com.readytype.missing.app")
        .is_none());
}

#[test]
fn terminal_bundle_resolves_png() {
    let icon = match MacOsAppIconSource.fetch("com.apple.Terminal") {
        Some(value) => value,
        None => {
            eprintln!("skip: NSWorkspace could not resolve com.apple.Terminal");
            return;
        }
    };
    assert!(!icon.png.is_empty());
    assert!(icon.mtime_secs > 0);
}
