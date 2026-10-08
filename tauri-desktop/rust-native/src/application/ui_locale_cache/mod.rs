//! Lock-free cache for the **resolved** UI / tray locale.
//!
//! Preference (`system`) is stored in settings; this cache only holds the
//! concrete language used for tray labels and never re-enters the switcher lock.
//! Locale codes are indexed from `tray.json` keys (sorted).

use std::sync::atomic::{AtomicU8, Ordering};

use super::tray_strings::{self, DEFAULT_RESOLVED_UI_LOCALE};

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
    let index = tray_strings::index_of(locale).unwrap_or_else(tray_strings::default_resolved_index);
    u8::try_from(index).expect("ui locale count fits in u8")
}

fn decode(code: u8) -> &'static str {
    tray_strings::code_at(usize::from(code)).unwrap_or(DEFAULT_RESOLVED_UI_LOCALE)
}

#[cfg(test)]
mod tests;
