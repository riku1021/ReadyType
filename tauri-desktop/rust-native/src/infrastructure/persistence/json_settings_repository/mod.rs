use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    application::{
        ports::settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
        settings::SwitcherSettings,
    },
    domain::input_source::{
        AxAttribute, FocusRule, InputSourceId, LanguageTag, MatchOperator, Platform, RuleCondition,
        RuleTarget, SearchScope,
    },
};

#[derive(Debug)]
pub struct JsonSettingsRepository {
    path: PathBuf,
    /// Set when corrupt load could not restore from `.bak`; blocks persisting defaults.
    writes_suspended: AtomicBool,
}

impl JsonSettingsRepository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            writes_suspended: AtomicBool::new(false),
        }
    }

    fn bak_path(&self) -> PathBuf {
        self.path.with_extension("bak.json")
    }

    fn corrupt_backup_path(&self) -> PathBuf {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_millis());
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let stem = self
            .path
            .file_stem()
            .and_then(|value| value.to_str())
            .unwrap_or("settings");
        parent.join(format!("{stem}.corrupt-{timestamp}.json"))
    }

    fn parse_settings_bytes(&self, bytes: &[u8]) -> Result<SwitcherSettings, String> {
        let decoded = serde_json::from_slice::<SettingsDto>(bytes).map_err(|e| e.to_string())?;
        let settings = SwitcherSettings::try_from(decoded).map_err(|e| e.to_string())?;
        settings.validate().map_err(|e| e.to_string())?;
        Ok(settings)
    }

    fn try_load_bak(&self) -> Option<SwitcherSettings> {
        let bytes = fs::read(self.bak_path()).ok()?;
        self.parse_settings_bytes(&bytes).ok()
    }

    fn snapshot_bak_best_effort(&self) {
        if !self.path.exists() {
            return;
        }
        let _ = fs::copy(&self.path, self.bak_path());
    }

    fn write_atomic(&self, bytes: &[u8]) -> Result<(), SettingsError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|error| SettingsError::Io(error.to_string()))?;
        }

        let temporary_path = self.path.with_extension("json.tmp");
        let write_result = (|| -> Result<(), SettingsError> {
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&temporary_path)
                .map_err(|error| SettingsError::Io(error.to_string()))?;
            file.write_all(bytes)
                .map_err(|error| SettingsError::Io(error.to_string()))?;
            file.sync_all()
                .map_err(|error| SettingsError::Io(error.to_string()))?;
            fs::rename(&temporary_path, &self.path)
                .map_err(|error| SettingsError::Io(error.to_string()))?;
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temporary_path);
        }
        write_result
    }

    /// Quarantine unreadable main; restore from `.bak` when possible, else suspend writes.
    fn recover(&self, reason: &str) -> Result<LoadOutcome, SettingsError> {
        let corrupt_path = self.corrupt_backup_path();
        fs::rename(&self.path, &corrupt_path)
            .map_err(|error| SettingsError::Io(error.to_string()))?;

        if let Some(settings) = self.try_load_bak() {
            let bytes = serde_json::to_vec_pretty(&SettingsDto::from(&settings))
                .map_err(|error| SettingsError::Serialization(error.to_string()))?;
            // Heal main without rotating `.bak` (keep last-known-good intact).
            self.write_atomic(&bytes)?;
            self.writes_suspended.store(false, Ordering::SeqCst);
            return Ok(LoadOutcome::RestoredFromBackup {
                settings,
                corrupt_path: corrupt_path.clone(),
                warning: format!(
                    "設定ファイルをバックアップから修復しました（{reason}）。破損コピー: {}",
                    corrupt_path.display()
                ),
            });
        }

        self.writes_suspended.store(true, Ordering::SeqCst);
        Ok(LoadOutcome::Recovered {
            settings: SwitcherSettings::defaults(),
            backup_path: corrupt_path,
            warning: format!(
                "設定ファイルを読み込めませんでした: {reason}。書き込みは抑制されています。"
            ),
        })
    }
}

impl SettingsRepository for JsonSettingsRepository {
    fn load(&self) -> Result<LoadOutcome, SettingsError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                self.writes_suspended.store(false, Ordering::SeqCst);
                return Ok(LoadOutcome::Defaults {
                    settings: SwitcherSettings::defaults(),
                });
            }
            Err(error) => return Err(SettingsError::Io(error.to_string())),
        };

        match self.parse_settings_bytes(&bytes) {
            Ok(settings) => {
                self.writes_suspended.store(false, Ordering::SeqCst);
                Ok(LoadOutcome::Loaded { settings })
            }
            Err(reason) => self.recover(&reason),
        }
    }

    fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError> {
        if self.writes_suspended.load(Ordering::SeqCst) {
            return Err(SettingsError::WritesSuspended);
        }
        settings
            .validate()
            .map_err(|error| SettingsError::InvalidSettings(error.to_string()))?;
        let bytes = serde_json::to_vec_pretty(&SettingsDto::from(settings))
            .map_err(|error| SettingsError::Serialization(error.to_string()))?;
        self.snapshot_bak_best_effort();
        self.write_atomic(&bytes)
    }
}

fn default_ui_locale() -> String {
    crate::application::settings::DEFAULT_UI_LOCALE_PREFERENCE.to_owned()
}

fn default_quick_set_shortcut() -> String {
    crate::application::settings::DEFAULT_QUICK_SET_SHORTCUT.to_owned()
}

/// 既存の設定ファイル（フィールドなし）では初回オンボーディングを強制しない。
fn default_onboarding_completed() -> bool {
    true
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDto {
    schema_version: u32,
    enabled: bool,
    /// New form: language tag → input source id (null = unset).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    language_preferences: Option<std::collections::BTreeMap<String, Option<String>>>,
    /// Legacy fields kept for read compatibility.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preferred_english: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    preferred_japanese: Option<String>,
    rules: Vec<RuleDto>,
    snoozed_until_unix_ms: Option<u64>,
    #[serde(default = "default_ui_locale")]
    ui_locale: String,
    #[serde(default = "default_quick_set_shortcut")]
    quick_set_shortcut: String,
    #[serde(default = "default_onboarding_completed")]
    onboarding_completed: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    input_source_display_names: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuleDto {
    id: String,
    name: String,
    enabled: bool,
    platform: PlatformDto,
    app_id: String,
    priority: u32,
    conditions: Vec<ConditionDto>,
    target: TargetDto,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ConditionDto {
    attribute: AttributeDto,
    operator: OperatorDto,
    scope: ScopeDto,
    value: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PlatformDto {
    Macos,
    Windows,
    Linux,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum AttributeDto {
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum OperatorDto {
    Equals,
    Contains,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ScopeDto {
    FocusedOnly,
    FocusedAndAncestors,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
enum TargetDto {
    PreferredLanguage {
        language: String,
    },
    /// Legacy read-only variants.
    PreferredEnglish,
    PreferredJapanese,
    Exact {
        #[serde(rename = "inputSourceId")]
        input_source_id: String,
    },
}

impl From<&SwitcherSettings> for SettingsDto {
    fn from(settings: &SwitcherSettings) -> Self {
        let language_preferences = settings
            .language_preferences
            .iter()
            .map(|(tag, id)| {
                (
                    tag.as_str().to_owned(),
                    id.as_ref().map(|value| value.as_str().to_owned()),
                )
            })
            .collect();
        Self {
            schema_version: settings.schema_version,
            enabled: settings.enabled,
            language_preferences: Some(language_preferences),
            preferred_english: None,
            preferred_japanese: None,
            rules: settings.rules.iter().map(RuleDto::from).collect(),
            snoozed_until_unix_ms: settings.snoozed_until_unix_ms,
            ui_locale: settings.ui_locale.clone(),
            quick_set_shortcut: settings.quick_set_shortcut.clone(),
            onboarding_completed: settings.onboarding_completed,
            input_source_display_names: settings.input_source_display_names.clone(),
        }
    }
}

impl TryFrom<SettingsDto> for SwitcherSettings {
    type Error = SettingsError;

    fn try_from(dto: SettingsDto) -> Result<Self, Self::Error> {
        let language_preferences = parse_language_preferences(&dto)?;
        let quick_set_shortcut =
            crate::application::quick_set_shortcut::normalize_quick_set_shortcut(
                &dto.quick_set_shortcut,
            )
            .map_err(invalid_settings)?;
        let settings = Self {
            schema_version: dto.schema_version,
            enabled: dto.enabled,
            language_preferences,
            rules: dto
                .rules
                .into_iter()
                .map(FocusRule::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            snoozed_until_unix_ms: dto.snoozed_until_unix_ms,
            ui_locale: SwitcherSettings::normalize_ui_locale(&dto.ui_locale),
            quick_set_shortcut,
            onboarding_completed: dto.onboarding_completed,
            input_source_display_names: dto.input_source_display_names,
        };
        settings.validate().map_err(invalid_settings)?;
        Ok(settings)
    }
}

fn parse_language_preferences(
    dto: &SettingsDto,
) -> Result<std::collections::BTreeMap<LanguageTag, Option<InputSourceId>>, SettingsError> {
    let mut preferences = std::collections::BTreeMap::new();

    if let Some(map) = &dto.language_preferences {
        for (tag, id) in map {
            let language = LanguageTag::parse(tag).map_err(invalid_settings)?;
            let input_source = match id {
                Some(value) => Some(InputSourceId::parse(value).map_err(invalid_settings)?),
                None => None,
            };
            preferences.insert(language, input_source);
        }
    }

    // Legacy fields fill missing keys (legacy-only files, or mixed documents).
    insert_legacy_preference(
        &mut preferences,
        LanguageTag::en(),
        dto.preferred_english.as_deref(),
    )?;
    insert_legacy_preference(
        &mut preferences,
        LanguageTag::ja(),
        dto.preferred_japanese.as_deref(),
    )?;

    Ok(preferences)
}

fn insert_legacy_preference(
    preferences: &mut std::collections::BTreeMap<LanguageTag, Option<InputSourceId>>,
    tag: LanguageTag,
    legacy_id: Option<&str>,
) -> Result<(), SettingsError> {
    let Some(id) = legacy_id else {
        return Ok(());
    };
    if preferences.contains_key(&tag) {
        return Ok(());
    }
    preferences.insert(
        tag,
        Some(InputSourceId::parse(id).map_err(invalid_settings)?),
    );
    Ok(())
}

impl From<&FocusRule> for RuleDto {
    fn from(rule: &FocusRule) -> Self {
        Self {
            id: rule.id().to_owned(),
            name: rule.name().to_owned(),
            enabled: rule.enabled(),
            platform: PlatformDto::from(rule.platform()),
            app_id: rule.app_id().to_owned(),
            priority: rule.priority(),
            conditions: rule.conditions().iter().map(ConditionDto::from).collect(),
            target: TargetDto::from(rule.target()),
        }
    }
}

impl TryFrom<RuleDto> for FocusRule {
    type Error = SettingsError;

    fn try_from(dto: RuleDto) -> Result<Self, Self::Error> {
        FocusRule::new(
            &dto.id,
            &dto.name,
            dto.enabled,
            Platform::from(dto.platform),
            &dto.app_id,
            dto.priority,
            dto.conditions
                .into_iter()
                .map(RuleCondition::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            RuleTarget::try_from(dto.target)?,
        )
        .map_err(invalid_settings)
    }
}

impl From<&RuleCondition> for ConditionDto {
    fn from(condition: &RuleCondition) -> Self {
        Self {
            attribute: AttributeDto::from(condition.attribute()),
            operator: OperatorDto::from(condition.operator()),
            scope: ScopeDto::from(condition.scope()),
            value: condition.value().to_owned(),
        }
    }
}

impl TryFrom<ConditionDto> for RuleCondition {
    type Error = SettingsError;

    fn try_from(dto: ConditionDto) -> Result<Self, Self::Error> {
        RuleCondition::new(
            AxAttribute::from(dto.attribute),
            MatchOperator::from(dto.operator),
            SearchScope::from(dto.scope),
            &dto.value,
        )
        .map_err(invalid_settings)
    }
}

impl From<Platform> for PlatformDto {
    fn from(value: Platform) -> Self {
        match value {
            Platform::MacOs => Self::Macos,
            Platform::Windows => Self::Windows,
            Platform::Linux => Self::Linux,
        }
    }
}

impl From<PlatformDto> for Platform {
    fn from(value: PlatformDto) -> Self {
        match value {
            PlatformDto::Macos => Self::MacOs,
            PlatformDto::Windows => Self::Windows,
            PlatformDto::Linux => Self::Linux,
        }
    }
}

impl From<AxAttribute> for AttributeDto {
    fn from(value: AxAttribute) -> Self {
        match value {
            AxAttribute::Role => Self::Role,
            AxAttribute::Subrole => Self::Subrole,
            AxAttribute::RoleDescription => Self::RoleDescription,
            AxAttribute::Title => Self::Title,
            AxAttribute::Description => Self::Description,
            AxAttribute::Identifier => Self::Identifier,
            AxAttribute::DomIdentifier => Self::DomIdentifier,
            AxAttribute::DomClassList => Self::DomClassList,
            AxAttribute::Help => Self::Help,
            AxAttribute::Placeholder => Self::Placeholder,
        }
    }
}

impl From<AttributeDto> for AxAttribute {
    fn from(value: AttributeDto) -> Self {
        match value {
            AttributeDto::Role => Self::Role,
            AttributeDto::Subrole => Self::Subrole,
            AttributeDto::RoleDescription => Self::RoleDescription,
            AttributeDto::Title => Self::Title,
            AttributeDto::Description => Self::Description,
            AttributeDto::Identifier => Self::Identifier,
            AttributeDto::DomIdentifier => Self::DomIdentifier,
            AttributeDto::DomClassList => Self::DomClassList,
            AttributeDto::Help => Self::Help,
            AttributeDto::Placeholder => Self::Placeholder,
        }
    }
}

impl From<MatchOperator> for OperatorDto {
    fn from(value: MatchOperator) -> Self {
        match value {
            MatchOperator::Equals => Self::Equals,
            MatchOperator::Contains => Self::Contains,
        }
    }
}

impl From<OperatorDto> for MatchOperator {
    fn from(value: OperatorDto) -> Self {
        match value {
            OperatorDto::Equals => Self::Equals,
            OperatorDto::Contains => Self::Contains,
        }
    }
}

impl From<SearchScope> for ScopeDto {
    fn from(value: SearchScope) -> Self {
        match value {
            SearchScope::FocusedOnly => Self::FocusedOnly,
            SearchScope::FocusedAndAncestors => Self::FocusedAndAncestors,
        }
    }
}

impl From<ScopeDto> for SearchScope {
    fn from(value: ScopeDto) -> Self {
        match value {
            ScopeDto::FocusedOnly => Self::FocusedOnly,
            ScopeDto::FocusedAndAncestors => Self::FocusedAndAncestors,
        }
    }
}

impl From<&RuleTarget> for TargetDto {
    fn from(value: &RuleTarget) -> Self {
        match value {
            RuleTarget::PreferredLanguage(tag) => Self::PreferredLanguage {
                language: tag.as_str().to_owned(),
            },
            RuleTarget::Exact(id) => Self::Exact {
                input_source_id: id.as_str().to_owned(),
            },
        }
    }
}

impl TryFrom<TargetDto> for RuleTarget {
    type Error = SettingsError;

    fn try_from(value: TargetDto) -> Result<Self, Self::Error> {
        match value {
            TargetDto::PreferredLanguage { language } => LanguageTag::parse(&language)
                .map(Self::PreferredLanguage)
                .map_err(invalid_settings),
            TargetDto::PreferredEnglish => Ok(Self::PreferredLanguage(LanguageTag::en())),
            TargetDto::PreferredJapanese => Ok(Self::PreferredLanguage(LanguageTag::ja())),
            TargetDto::Exact { input_source_id } => InputSourceId::parse(&input_source_id)
                .map(Self::Exact)
                .map_err(invalid_settings),
        }
    }
}

fn invalid_settings(error: impl ToString) -> SettingsError {
    SettingsError::InvalidSettings(error.to_string())
}

#[cfg(test)]
mod tests;
