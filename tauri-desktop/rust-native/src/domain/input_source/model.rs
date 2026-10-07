use std::collections::{BTreeMap, BTreeSet};

use crate::domain::errors::DomainError;

fn required(value: &str, field: &str) -> Result<String, DomainError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(DomainError::InvalidInput(format!(
            "{field} must not be empty"
        )));
    }
    Ok(trimmed.to_owned())
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputSourceId(String);

impl InputSourceId {
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        required(value, "input source id").map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// BCP-47 primary language subtag (open string, not a closed union).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LanguageTag(String);

impl LanguageTag {
    pub fn parse(value: &str) -> Result<Self, DomainError> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidInput(
                "language tag must not be empty".to_owned(),
            ));
        }
        let primary = trimmed
            .split(['-', '_'])
            .next()
            .unwrap_or(trimmed)
            .to_ascii_lowercase();
        if primary.is_empty() {
            return Err(DomainError::InvalidInput(
                "language tag must not be empty".to_owned(),
            ));
        }
        Ok(Self(primary))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn en() -> Self {
        Self("en".to_owned())
    }

    pub fn ja() -> Self {
        Self("ja".to_owned())
    }
}

/// IME のラテン／英数モードか（Apple / Google が使う `.Roman` サフィックス）。
/// 言語名は見ない。`.FullWidthRoman` は全角英数で別扱い。
pub fn is_ime_latin_mode(source_id: &str) -> bool {
    let id = source_id.to_ascii_lowercase();
    if id.ends_with(".fullwidthroman") {
        return false;
    }
    id.ends_with(".roman")
}

fn parent_input_source_prefix(source_id: &str) -> Option<&str> {
    source_id.rsplit_once('.').map(|(prefix, _)| prefix)
}

/// TIS が IME 英数モードを `en` と報告するケースを、同一プレフィックス家族の言語へ寄せる。
/// 家族内の非 `en` がただ一つのときだけ書き換え（言語ホワイトリストなし）。
pub fn canonicalize_language_codes_by_family(entries: &mut [(String, Option<String>)]) {
    let mut family_non_en: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (id, language) in entries.iter() {
        let Some(prefix) = parent_input_source_prefix(id.as_str()) else {
            continue;
        };
        let Some(code) = language.as_deref() else {
            continue;
        };
        let Ok(tag) = LanguageTag::parse(code) else {
            continue;
        };
        if tag.as_str() == "en" {
            continue;
        }
        family_non_en
            .entry(prefix.to_owned())
            .or_default()
            .insert(tag.as_str().to_owned());
    }

    for (id, language) in entries.iter_mut() {
        if !is_ime_latin_mode(id.as_str()) {
            continue;
        }
        let reported_is_en = language
            .as_deref()
            .and_then(|code| LanguageTag::parse(code).ok())
            .is_some_and(|tag| tag.as_str() == "en");
        if !reported_is_en {
            continue;
        }
        let Some(prefix) = parent_input_source_prefix(id.as_str()) else {
            continue;
        };
        let Some(non_en) = family_non_en.get(prefix) else {
            continue;
        };
        if non_en.len() != 1 {
            continue;
        }
        if let Some(only) = non_en.iter().next() {
            *language = Some(only.clone());
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Platform {
    MacOs,
    Windows,
    Linux,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformAppId {
    platform: Platform,
    app_id: String,
    pid: i32,
}

impl PlatformAppId {
    pub fn new(platform: Platform, app_id: &str, pid: i32) -> Result<Self, DomainError> {
        Ok(Self {
            platform,
            app_id: required(app_id, "application id")?,
            pid,
        })
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn pid(&self) -> i32 {
        self.pid
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AxAttribute {
    Role,
    Subrole,
    RoleDescription,
    Title,
    Description,
    Identifier,
    DomIdentifier,
    DomClassList,
    Help,
    Placeholder,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AxElementSnapshot {
    role: String,
    subrole: String,
    role_description: String,
    title: String,
    description: String,
    identifier: String,
    dom_identifier: String,
    dom_class_list: String,
    help: String,
    placeholder: String,
}

impl AxElementSnapshot {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with(mut self, attribute: AxAttribute, value: impl Into<String>) -> Self {
        let target = match attribute {
            AxAttribute::Role => &mut self.role,
            AxAttribute::Subrole => &mut self.subrole,
            AxAttribute::RoleDescription => &mut self.role_description,
            AxAttribute::Title => &mut self.title,
            AxAttribute::Description => &mut self.description,
            AxAttribute::Identifier => &mut self.identifier,
            AxAttribute::DomIdentifier => &mut self.dom_identifier,
            AxAttribute::DomClassList => &mut self.dom_class_list,
            AxAttribute::Help => &mut self.help,
            AxAttribute::Placeholder => &mut self.placeholder,
        };
        *target = value.into();
        self
    }

    pub fn value(&self, attribute: AxAttribute) -> &str {
        match attribute {
            AxAttribute::Role => &self.role,
            AxAttribute::Subrole => &self.subrole,
            AxAttribute::RoleDescription => &self.role_description,
            AxAttribute::Title => &self.title,
            AxAttribute::Description => &self.description,
            AxAttribute::Identifier => &self.identifier,
            AxAttribute::DomIdentifier => &self.dom_identifier,
            AxAttribute::DomClassList => &self.dom_class_list,
            AxAttribute::Help => &self.help,
            AxAttribute::Placeholder => &self.placeholder,
        }
    }
}

/// fingerprint / focus_key が空フォーカスを表すときの値。
pub const EMPTY_FOCUS_KEY: &str = "empty";
/// focus identity に使う祖先数（それ以深の Dom* / Identifier は key に含めない）。
pub const FOCUS_IDENTITY_DEPTH: usize = 6;

/// IME / 装飾で付く揮発クラス。identity からは除き、ルールマッチ用の生 class は触らない。
fn is_volatile_dom_class_token(token: &str) -> bool {
    matches!(
        token,
        "ime-input" | "ProseMirror-focused" | "focused" | "focus-visible"
    ) || token.ends_with("-focused")
}

/// DomClassList を identity 用に正規化（揮発トークン除去・ソート）。
pub fn normalize_dom_class_list_for_identity(class_list: &str) -> String {
    let mut tokens: Vec<&str> = class_list
        .split_whitespace()
        .filter(|token| !token.is_empty() && !is_volatile_dom_class_token(token))
        .collect();
    tokens.sort_unstable();
    tokens.dedup();
    tokens.join(" ")
}

/// フォーカス文脈の安定キー（**focused 要素のみ**）。
///
/// 祖先チェーンは Electron で長さ・class が揺れやすく、入力中の偽フォーカス変化の主因になる。
/// composer / terminal / monaco は focused の Role + DomClassList で区別できる。
pub fn focus_identity_key(elements: &[AxElementSnapshot]) -> String {
    let Some(focused) = elements.first() else {
        return EMPTY_FOCUS_KEY.to_owned();
    };
    format!(
        "{}`{}`{}`{}",
        focused.value(AxAttribute::Role),
        focused.value(AxAttribute::DomIdentifier),
        normalize_dom_class_list_for_identity(focused.value(AxAttribute::DomClassList)),
        focused.value(AxAttribute::Identifier),
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    app: PlatformAppId,
    generation: u64,
    elements: Vec<AxElementSnapshot>,
    focus_key: String,
}

impl FocusSnapshot {
    pub fn new(app: PlatformAppId, generation: u64, elements: Vec<AxElementSnapshot>) -> Self {
        let focus_key = focus_identity_key(&elements);
        Self {
            app,
            generation,
            elements,
            focus_key,
        }
    }

    pub fn app(&self) -> &PlatformAppId {
        &self.app
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }

    pub fn elements(&self) -> &[AxElementSnapshot] {
        &self.elements
    }

    /// 正規化済みフォーカス文脈キー（手動 override の紐づけに使う）。
    pub fn focus_key(&self) -> &str {
        &self.focus_key
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchOperator {
    Equals,
    Contains,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchScope {
    FocusedOnly,
    FocusedAndAncestors,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleCondition {
    attribute: AxAttribute,
    operator: MatchOperator,
    scope: SearchScope,
    value: String,
}

impl RuleCondition {
    pub fn new(
        attribute: AxAttribute,
        operator: MatchOperator,
        scope: SearchScope,
        value: &str,
    ) -> Result<Self, DomainError> {
        Ok(Self {
            attribute,
            operator,
            scope,
            value: required(value, "condition value")?,
        })
    }

    pub fn attribute(&self) -> AxAttribute {
        self.attribute
    }

    pub fn operator(&self) -> MatchOperator {
        self.operator
    }

    pub fn scope(&self) -> SearchScope {
        self.scope
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuleTarget {
    PreferredLanguage(LanguageTag),
    Exact(InputSourceId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusRule {
    id: String,
    name: String,
    enabled: bool,
    platform: Platform,
    app_id: String,
    priority: u32,
    conditions: Vec<RuleCondition>,
    target: RuleTarget,
}

impl FocusRule {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        name: &str,
        enabled: bool,
        platform: Platform,
        app_id: &str,
        priority: u32,
        conditions: Vec<RuleCondition>,
        target: RuleTarget,
    ) -> Result<Self, DomainError> {
        let rule = Self {
            id: required(id, "rule id")?,
            name: required(name, "rule name")?,
            enabled,
            platform,
            app_id: required(app_id, "rule application id")?,
            priority,
            conditions,
            target,
        };
        rule.validate()?;
        Ok(rule)
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        required(&self.id, "rule id")?;
        required(&self.name, "rule name")?;
        required(&self.app_id, "rule application id")?;
        for condition in &self.conditions {
            required(condition.value(), "condition value")?;
        }
        Ok(())
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn platform(&self) -> Platform {
        self.platform
    }

    pub fn app_id(&self) -> &str {
        &self.app_id
    }

    pub fn priority(&self) -> u32 {
        self.priority
    }

    pub fn conditions(&self) -> &[RuleCondition] {
        &self.conditions
    }

    pub fn target(&self) -> &RuleTarget {
        &self.target
    }
}

#[cfg(test)]
mod focus_identity_tests {
    use super::*;

    #[test]
    fn normalize_dom_class_strips_volatile_tokens_and_sorts() {
        let normalized = normalize_dom_class_list_for_identity(
            "ime-input monaco-mouse-cursor-text inputarea ProseMirror-focused",
        );
        assert_eq!(normalized, "inputarea monaco-mouse-cursor-text");
    }

    #[test]
    fn focus_identity_key_stable_across_ime_class() {
        let base = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomClassList, "aislash-editor-input")];
        let with_ime = [AxElementSnapshot::new()
            .with(AxAttribute::Role, "AXTextArea")
            .with(AxAttribute::DomClassList, "aislash-editor-input ime-input")];
        assert_eq!(focus_identity_key(&base), focus_identity_key(&with_ime));
        assert_ne!(focus_identity_key(&[]), focus_identity_key(&base));
    }

    #[test]
    fn focus_identity_key_ignores_ancestor_dom_class_churn() {
        let stable = [
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextArea")
                .with(AxAttribute::DomClassList, "aislash-editor-input"),
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXGroup")
                .with(AxAttribute::DomClassList, "split-view-view"),
        ];
        let churned = [
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXTextArea")
                .with(AxAttribute::DomClassList, "aislash-editor-input"),
            AxElementSnapshot::new()
                .with(AxAttribute::Role, "AXGroup")
                .with(AxAttribute::DomClassList, "split-view-view visible"),
        ];
        assert_eq!(focus_identity_key(&stable), focus_identity_key(&churned));
    }
}
