//! Lock-free cache for the **resolved** UI / tray locale (`ja` | `en`).
//!
//! Preference (`system`) is stored in settings; this cache only holds the
//! concrete language used for tray labels and never re-enters the switcher lock.

use std::sync::atomic::{AtomicU8, Ordering};

use super::settings::DEFAULT_RESOLVED_UI_LOCALE;

const LOCALE_JA: u8 = 0;
const LOCALE_EN: u8 = 1;

#[derive(Debug)]
pub struct UiLocaleCache {
    code: AtomicU8,
}

impl UiLocaleCache {
    pub fn new(resolved_locale: &str) -> Self {
        Self {
            code: AtomicU8::new(encode(resolved_locale)),
        }
    }

    pub fn store(&self, resolved_locale: &str) {
        self.code.store(encode(resolved_locale), Ordering::Release);
    }

    pub fn load(&self) -> String {
        decode(self.code.load(Ordering::Acquire)).to_owned()
    }
}

fn encode(locale: &str) -> u8 {
    if locale == "en" {
        LOCALE_EN
    } else {
        LOCALE_JA
    }
}

fn decode(code: u8) -> &'static str {
    if code == LOCALE_EN {
        "en"
    } else {
        DEFAULT_RESOLVED_UI_LOCALE
    }
}

#[cfg(test)]
mod tests {
    use super::UiLocaleCache;

    #[test]
    fn stores_and_loads_resolved_locales() {
        let cache = UiLocaleCache::new("ja");
        assert_eq!(cache.load(), "ja");
        cache.store("en");
        assert_eq!(cache.load(), "en");
        cache.store("system");
        assert_eq!(cache.load(), "ja");
    }
}
