use std::{
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
    debug_logging: bool,
    snoozed_until_unix_ms: Option<u64>,
    #[serde(default = "default_ui_locale")]
    ui_locale: String,
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
            debug_logging: settings.debug_logging,
            snoozed_until_unix_ms: settings.snoozed_until_unix_ms,
            ui_locale: settings.ui_locale.clone(),
        }
    }
}

impl TryFrom<SettingsDto> for SwitcherSettings {
    type Error = SettingsError;

    fn try_from(dto: SettingsDto) -> Result<Self, Self::Error> {
        let language_preferences = parse_language_preferences(&dto)?;
        let settings = Self {
            schema_version: dto.schema_version,
            enabled: dto.enabled,
            language_preferences,
            rules: dto
                .rules
                .into_iter()
                .map(FocusRule::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            debug_logging: dto.debug_logging,
            snoozed_until_unix_ms: dto.snoozed_until_unix_ms,
            ui_locale: SwitcherSettings::normalize_ui_locale(&dto.ui_locale),
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
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
        sync::atomic::{AtomicU64, Ordering},
    };

    use super::JsonSettingsRepository;
    use crate::application::{
        ports::settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
        settings::SwitcherSettings,
    };

    static NEXT_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

    struct TestDirectory(PathBuf);

    impl TestDirectory {
        fn new(name: &str) -> Self {
            let id = NEXT_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "readytype-settings-{name}-{}-{id}",
                std::process::id()
            ));
            fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }

        fn join(&self, name: &str) -> PathBuf {
            self.0.join(name)
        }
    }

    impl Drop for TestDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn repository(directory: &TestDirectory) -> JsonSettingsRepository {
        JsonSettingsRepository::new(directory.join("settings.json"))
    }

    fn write_settings(path: &Path, contents: &str) {
        fs::write(path, contents).expect("write test settings");
    }

    #[test]
    fn save_then_load_round_trips() {
        let directory = TestDirectory::new("round-trip");
        let repository = repository(&directory);
        let expected = SwitcherSettings::defaults();

        repository.save(&expected).expect("save");
        let outcome = repository.load().expect("load");

        assert!(matches!(outcome, LoadOutcome::Loaded { .. }));
        assert_eq!(outcome.settings(), &expected);
    }

    #[test]
    fn missing_file_returns_defaults_without_creating_a_file() {
        let directory = TestDirectory::new("missing");
        let repository = repository(&directory);

        let outcome = repository.load().expect("load defaults");

        assert!(matches!(outcome, LoadOutcome::Defaults { .. }));
        assert_eq!(outcome.settings(), &SwitcherSettings::defaults());
        assert!(!directory.join("settings.json").exists());
    }

    #[test]
    fn schema_v1_legacy_json_decodes_into_language_preferences() {
        let directory = TestDirectory::new("schema-v1");
        let path = directory.join("settings.json");
        write_settings(
            &path,
            r#"{
                "schemaVersion": 1,
                "enabled": false,
                "preferredEnglish": "english.id",
                "preferredJapanese": "japanese.id",
                "rules": [],
                "debugLogging": true,
                "snoozedUntilUnixMs": 1234
            }"#,
        );

        let outcome = repository(&directory).load().expect("decode schema v1");

        assert!(!outcome.settings().enabled);
        assert_eq!(
            outcome
                .settings()
                .preferred(&crate::domain::input_source::LanguageTag::en())
                .map(|id| id.as_str()),
            Some("english.id")
        );
        assert_eq!(
            outcome
                .settings()
                .preferred(&crate::domain::input_source::LanguageTag::ja())
                .map(|id| id.as_str()),
            Some("japanese.id")
        );
        assert!(outcome.settings().debug_logging);
        assert_eq!(outcome.settings().snoozed_until_unix_ms, Some(1234));
    }

    #[test]
    fn language_preferences_form_decodes_and_legacy_targets_migrate() {
        let directory = TestDirectory::new("language-preferences");
        let path = directory.join("settings.json");
        write_settings(
            &path,
            r#"{
                "schemaVersion": 1,
                "enabled": true,
                "languagePreferences": {
                    "en": "english.id",
                    "ja": "japanese.id"
                },
                "rules": [{
                    "id": "legacy-en",
                    "name": "Legacy EN",
                    "enabled": true,
                    "platform": "macos",
                    "appId": "com.example",
                    "priority": 0,
                    "conditions": [],
                    "target": { "kind": "preferredEnglish" }
                }],
                "debugLogging": false,
                "snoozedUntilUnixMs": null
            }"#,
        );

        let outcome = repository(&directory)
            .load()
            .expect("decode language preferences");
        let settings = outcome.settings();
        assert_eq!(
            settings
                .preferred(&crate::domain::input_source::LanguageTag::en())
                .map(|id| id.as_str()),
            Some("english.id")
        );
        assert_eq!(
            settings.rules[0].target(),
            &crate::domain::input_source::RuleTarget::PreferredLanguage(
                crate::domain::input_source::LanguageTag::en()
            )
        );

        repository(&directory).save(settings).expect("rewrite");
        let rewritten = fs::read_to_string(path).expect("read rewritten");
        assert!(rewritten.contains("languagePreferences"));
        assert!(rewritten.contains("preferredLanguage"));
        assert!(!rewritten.contains("preferredEnglish"));
    }

    #[test]
    fn corrupt_json_is_backed_up_and_defaults_are_returned() {
        let directory = TestDirectory::new("corrupt");
        let path = directory.join("settings.json");
        write_settings(&path, "{not-json");

        let repo = repository(&directory);
        let outcome = repo.load().expect("recover");

        assert!(matches!(outcome, LoadOutcome::Recovered { .. }));
        assert!(outcome.writes_suspended());
        assert_eq!(outcome.settings(), &SwitcherSettings::defaults());
        assert!(outcome.backup_path().expect("backup path").exists());
        assert!(outcome
            .warning()
            .expect("warning")
            .contains("読み込めません"));
        assert!(!path.exists());
        assert!(matches!(
            repo.save(&SwitcherSettings::defaults()),
            Err(SettingsError::WritesSuspended)
        ));
    }

    #[test]
    fn corrupt_main_restores_from_bak_without_suspending_writes() {
        let directory = TestDirectory::new("restore-bak");
        let path = directory.join("settings.json");
        let bak = directory.join("settings.bak.json");
        let repo = repository(&directory);

        repo.save(&SwitcherSettings::defaults())
            .expect("first save");
        repo.save(&SwitcherSettings::defaults())
            .expect("second save creates bak");
        assert!(bak.exists());
        write_settings(&path, "{not-json");

        let outcome = repo.load().expect("restore");
        assert!(matches!(outcome, LoadOutcome::RestoredFromBackup { .. }));
        assert!(!outcome.writes_suspended());
        assert!(path.exists());
        assert!(outcome.backup_path().expect("corrupt path").exists());
        repo.save(&SwitcherSettings::defaults())
            .expect("writes remain allowed");
    }

    #[test]
    fn save_rotates_last_known_good_bak() {
        let directory = TestDirectory::new("bak-rotate");
        let path = directory.join("settings.json");
        let bak = directory.join("settings.bak.json");
        let repo = repository(&directory);

        let mut first = SwitcherSettings::defaults();
        first.debug_logging = true;
        repo.save(&first).expect("first save");
        assert!(!bak.exists(), "no prior main → no bak yet");

        let second = SwitcherSettings::defaults();
        repo.save(&second).expect("second save");
        assert!(bak.exists());
        let bak_text = fs::read_to_string(&bak).expect("read bak");
        assert!(bak_text.contains("\"debugLogging\": true"));
        let main_text = fs::read_to_string(&path).expect("read main");
        assert!(main_text.contains("\"debugLogging\": false"));
    }

    #[test]
    fn future_schema_is_backed_up_instead_of_being_silently_downgraded() {
        let directory = TestDirectory::new("future-schema");
        let path = directory.join("settings.json");
        write_settings(
            &path,
            r#"{
                "schemaVersion": 99,
                "enabled": true,
                "preferredEnglish": "english.id",
                "preferredJapanese": "japanese.id",
                "rules": [],
                "debugLogging": false,
                "snoozedUntilUnixMs": null
            }"#,
        );

        let outcome = repository(&directory)
            .load()
            .expect("recover future schema");

        assert!(matches!(outcome, LoadOutcome::Recovered { .. }));
        assert!(outcome.backup_path().expect("backup path").exists());
    }

    #[test]
    fn invalid_input_source_is_backed_up() {
        let directory = TestDirectory::new("invalid-input-source");
        let path = directory.join("settings.json");
        write_settings(
            &path,
            r#"{
                "schemaVersion": 1,
                "enabled": true,
                "preferredEnglish": "   ",
                "preferredJapanese": "japanese.id",
                "rules": [],
                "debugLogging": false,
                "snoozedUntilUnixMs": null
            }"#,
        );

        let outcome = repository(&directory)
            .load()
            .expect("recover invalid settings");

        assert!(matches!(outcome, LoadOutcome::Recovered { .. }));
        assert!(outcome.backup_path().expect("backup path").exists());
    }

    #[test]
    fn successful_save_leaves_no_temporary_file() {
        let directory = TestDirectory::new("temporary-file");
        let repository = repository(&directory);

        repository
            .save(&SwitcherSettings::defaults())
            .expect("save settings");

        assert!(!directory.join("settings.json.tmp").exists());
    }
}
