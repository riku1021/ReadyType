//! System icon appearance token + change monitoring for cache invalidation.
//!
//! Detection uses a lightweight background poll of `AppleIconAppearanceTheme`.
//! Full KVO on UserDefaults needs deferred reads (CFPrefs lock) and often a
//! forced-sync timer for cross-process writes; polling one string every second
//! keeps FE hot-path event-driven without that machinery.
//!
//! Emitting to the WebView is wired at the composition root (callback), so this
//! module stays free of Tauri types.

use std::sync::Arc;
use std::thread;
use std::time::Duration;

use objc2_foundation::{NSString, NSUserDefaults};

use crate::infrastructure::platform::app_icon_cache::IconStyleSource;

const POLL_INTERVAL: Duration = Duration::from_secs(1);

/// Reads macOS `AppleIconAppearanceTheme` (Default / Dark / Clear / Tinted, etc.).
pub struct MacOsIconStyleSource;

impl IconStyleSource for MacOsIconStyleSource {
    fn token(&self) -> String {
        current_icon_style_token()
    }
}

pub fn shared() -> Arc<dyn IconStyleSource> {
    Arc::new(MacOsIconStyleSource)
}

pub fn current_icon_style_token() -> String {
    let defaults = NSUserDefaults::standardUserDefaults();
    let key = NSString::from_str("AppleIconAppearanceTheme");
    defaults
        .stringForKey(&key)
        .map(|value| value.to_string())
        .unwrap_or_default()
}

/// Invokes `on_change(style_token)` when the system icon style token changes.
pub fn start_monitor(on_change: impl Fn(String) + Send + 'static) {
    let _ = thread::Builder::new()
        .name("readytype-icon-style".into())
        .spawn(move || {
            let mut last = current_icon_style_token();
            loop {
                thread::sleep(POLL_INTERVAL);
                let next = current_icon_style_token();
                if next == last {
                    continue;
                }
                last = next.clone();
                on_change(next);
            }
        });
}
