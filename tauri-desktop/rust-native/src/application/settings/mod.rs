use std::collections::BTreeMap;

use crate::application::tray_strings::{self, map_language_tag};
use crate::domain::{
    errors::DomainError,
    input_source::{FocusRule, InputSourceId, LanguageTag, RuleSet, RuleTarget},
};

pub use crate::application::quick_set_shortcut::DEFAULT_QUICK_SET_SHORTCUT;
pub use crate::application::tray_strings::{
    DEFAULT_RESOLVED_UI_LOCALE, DEFAULT_UI_LOCALE_PREFERENCE,
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitcherSettings {
    pub schema_version: u32,
    pub enabled: bool,
    /// Per-language preferred input source. `None` value = unset.
    /// Empty map = not yet seeded from OS (see `seed_language_preferences_from_sources`).
    pub language_preferences: BTreeMap<LanguageTag, Option<InputSourceId>>,
    pub rules: Vec<FocusRule>,
    pub snoozed_until_unix_ms: Option<u64>,
    /// UI language preference (`system` | tray.json pack code).
    pub ui_locale: String,
    /// Quick Set グローバルショートカット（Tauri accelerator 文字列）。
    pub quick_set_shortcut: String,
    /// 初回オンボーディングを完了したか。
    pub onboarding_completed: bool,
    /// Last-known LocalizedName per input source id (kept after OS removal for UI).
    pub input_source_display_names: BTreeMap<String, String>,
}

impl SwitcherSettings {
    pub fn defaults() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled: true,
            language_preferences: BTreeMap::new(),
            // 製品プリセットは置かない。既存ユーザーの保存済みルールはロードで維持される。
            rules: Vec::new(),
            snoozed_until_unix_ms: None,
            ui_locale: DEFAULT_UI_LOCALE_PREFERENCE.to_owned(),
            quick_set_shortcut: DEFAULT_QUICK_SET_SHORTCUT.to_owned(),
            onboarding_completed: false,
            input_source_display_names: BTreeMap::new(),
        }
    }

    pub fn preferred(&self, tag: &LanguageTag) -> Option<&InputSourceId> {
        self.language_preferences
            .get(tag)
            .and_then(|value| value.as_ref())
    }

    /// Normalize a stored preference to `system` or a tray-supported UI pack.
    pub fn normalize_ui_locale(locale: &str) -> String {
        if locale == "system" || tray_strings::is_supported_ui_locale(locale) {
            return locale.to_owned();
        }
        DEFAULT_UI_LOCALE_PREFERENCE.to_owned()
    }

    /// Resolve preference to a concrete UI language pack.
    pub fn resolve_ui_locale(preference: &str) -> String {
        let normalized = Self::normalize_ui_locale(preference);
        if tray_strings::is_supported_ui_locale(&normalized) {
            return normalized;
        }
        resolve_os_ui_locale()
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(DomainError::InvalidInput(format!(
                "unsupported settings schema version: {}",
                self.schema_version
            )));
        }
        RuleSet::new(self.rules.clone())?;
        crate::application::quick_set_shortcut::normalize_quick_set_shortcut(
            &self.quick_set_shortcut,
        )?;
        Ok(())
    }
}

/// OS 駆動の初期シード。言語ホワイトリストは持たない。
/// 現在の入力ソースの言語を 1 スロット。無ければ一覧の先頭言語グループを 1 つ。
pub fn seed_language_preferences_from_sources(
    sources: &[crate::application::ports::input_source::AvailableInputSource],
    current: Option<&InputSourceId>,
) -> BTreeMap<LanguageTag, Option<InputSourceId>> {
    let by_language = group_sources_by_language(sources);

    let mut preferences = BTreeMap::new();
    if let Some(current_id) = current {
        if let Some(current_source) = sources.iter().find(|source| &source.id == current_id) {
            if let Some(code) = current_source.language_code.as_deref() {
                if let Ok(tag) = LanguageTag::parse(code) {
                    if let Some(group) = by_language.get(&tag) {
                        preferences.insert(tag, pick_default_source_id(group));
                        return preferences;
                    }
                }
            }
        }
    }

    if let Some((tag, group)) = by_language.into_iter().next() {
        preferences.insert(tag, pick_default_source_id(&group));
    }
    preferences
}

/// Upsert display names from a successful OS list. Returns true if map changed.
///
/// Only non-empty names are upserted. Entries for ids missing from `sources` are
/// never removed (they must survive OS removal).
pub fn merge_input_source_display_names(
    names: &mut BTreeMap<String, String>,
    sources: &[crate::application::ports::input_source::AvailableInputSource],
) -> bool {
    let mut changed = false;
    for source in sources {
        if source.name.is_empty() || names.get(source.id.as_str()) == Some(&source.name) {
            continue;
        }
        names.insert(source.id.as_str().to_owned(), source.name.clone());
        changed = true;
    }
    changed
}

/// OS 一覧と突き合わせ、消失・言語不一致の preference を差し替える。
///
/// - 同言語の利用可能ソースがあれば差し替える
/// - 代替が無いときは設定済み ID を保持する（UI は利用不可＋警告バッジで示す）
/// - 明示 unset（`None`）は埋めない
pub fn reconcile_language_preferences(
    preferences: &BTreeMap<LanguageTag, Option<InputSourceId>>,
    sources: &[crate::application::ports::input_source::AvailableInputSource],
) -> BTreeMap<LanguageTag, Option<InputSourceId>> {
    let by_language = group_sources_by_language(sources);
    let mut result = BTreeMap::new();
    for (tag, preferred) in preferences {
        let still_valid = preferred.as_ref().is_some_and(|id| {
            sources.iter().any(|source| {
                source.available
                    && &source.id == id
                    && source
                        .language_code
                        .as_deref()
                        .and_then(|code| LanguageTag::parse(code).ok())
                        .as_ref()
                        == Some(tag)
            })
        });
        if still_valid {
            result.insert(tag.clone(), preferred.clone());
            continue;
        }
        let Some(configured_id) = preferred.clone() else {
            // 明示 unset は OS から埋め直さない
            result.insert(tag.clone(), None);
            continue;
        };
        let replacement = by_language
            .get(tag)
            .and_then(|group| pick_default_source_id(group));
        result.insert(tag.clone(), Some(replacement.unwrap_or(configured_id)));
    }
    result
}

/// ルールが参照する PreferredLanguage で、preference にキーが無い言語を OS から埋める。
/// 既存キー（明示的な unset 含む）は触らない。シードが 1 言語だけのとき ja ルール等が空解決になるのを防ぐ。
pub fn ensure_preferences_for_rule_languages(
    preferences: &BTreeMap<LanguageTag, Option<InputSourceId>>,
    sources: &[crate::application::ports::input_source::AvailableInputSource],
    rules: &[FocusRule],
) -> BTreeMap<LanguageTag, Option<InputSourceId>> {
    let by_language = group_sources_by_language(sources);
    let mut result = preferences.clone();
    for rule in rules {
        let RuleTarget::PreferredLanguage(tag) = rule.target() else {
            continue;
        };
        if result.contains_key(tag) {
            continue;
        }
        let picked = by_language
            .get(tag)
            .and_then(|group| pick_default_source_id(group));
        result.insert(tag.clone(), picked);
    }
    result
}

pub(crate) fn group_sources_by_language(
    sources: &[crate::application::ports::input_source::AvailableInputSource],
) -> BTreeMap<LanguageTag, Vec<&crate::application::ports::input_source::AvailableInputSource>> {
    let mut by_language: BTreeMap<
        LanguageTag,
        Vec<&crate::application::ports::input_source::AvailableInputSource>,
    > = BTreeMap::new();
    for source in sources {
        let Some(code) = source.language_code.as_deref() else {
            continue;
        };
        let Ok(tag) = LanguageTag::parse(code) else {
            continue;
        };
        by_language.entry(tag).or_default().push(source);
    }
    by_language
}

pub(crate) fn pick_default_source_id(
    group: &[&crate::application::ports::input_source::AvailableInputSource],
) -> Option<InputSourceId> {
    let available = group
        .iter()
        .copied()
        .filter(|source| source.available)
        .collect::<Vec<_>>();
    available
        .iter()
        .find(|source| !crate::domain::input_source::is_ime_latin_mode(source.id.as_str()))
        .or_else(|| available.first())
        .map(|source| source.id.clone())
}

fn resolve_os_ui_locale() -> String {
    for tag in sys_locale::get_locales() {
        if let Some(resolved) = map_language_tag(&tag) {
            return resolved.to_owned();
        }
    }
    if let Some(tag) = sys_locale::get_locale() {
        if let Some(resolved) = map_language_tag(&tag) {
            return resolved.to_owned();
        }
    }
    DEFAULT_RESOLVED_UI_LOCALE.to_owned()
}

#[cfg(test)]
mod tests;
