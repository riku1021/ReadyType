use std::{
    fs::{self, OpenOptions},
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use serde::{Deserialize, Serialize};

use crate::{
    application::{
        ports::settings_repository::{LoadOutcome, SettingsError, SettingsRepository},
        settings::SwitcherSettings,
    },
    domain::input_source::{
        AxAttribute, FocusRule, InputSourceId, MatchOperator, Platform, RuleCondition, RuleTarget,
        SearchScope,
    },
};

#[derive(Debug, Clone)]
pub struct JsonSettingsRepository {
    path: PathBuf,
}

impl JsonSettingsRepository {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn recover(&self, reason: &str) -> Result<LoadOutcome, SettingsError> {
        let backup_path = self.corrupt_backup_path();
        fs::rename(&self.path, &backup_path)
            .map_err(|error| SettingsError::Io(error.to_string()))?;
        Ok(LoadOutcome::Recovered {
            settings: SwitcherSettings::defaults(),
            backup_path,
            warning: format!("設定ファイルを読み込めませんでした: {reason}"),
        })
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
}

impl SettingsRepository for JsonSettingsRepository {
    fn load(&self) -> Result<LoadOutcome, SettingsError> {
        let bytes = match fs::read(&self.path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Ok(LoadOutcome::Defaults {
                    settings: SwitcherSettings::defaults(),
                });
            }
            Err(error) => return Err(SettingsError::Io(error.to_string())),
        };

        let decoded = match serde_json::from_slice::<SettingsDto>(&bytes) {
            Ok(decoded) => decoded,
            Err(error) => return self.recover(&error.to_string()),
        };
        let settings = match SwitcherSettings::try_from(decoded) {
            Ok(settings) => settings,
            Err(error) => return self.recover(&error.to_string()),
        };
        if let Err(error) = settings.validate() {
            return self.recover(&error.to_string());
        }

        Ok(LoadOutcome::Loaded { settings })
    }

    fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError> {
        settings
            .validate()
            .map_err(|error| SettingsError::InvalidSettings(error.to_string()))?;
        let bytes = serde_json::to_vec_pretty(&SettingsDto::from(settings))
            .map_err(|error| SettingsError::Serialization(error.to_string()))?;
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
            file.write_all(&bytes)
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
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SettingsDto {
    schema_version: u32,
    enabled: bool,
    preferred_english: String,
    preferred_japanese: String,
    rules: Vec<RuleDto>,
    debug_logging: bool,
    snoozed_until_unix_ms: Option<u64>,
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
    PreferredEnglish,
    PreferredJapanese,
    Exact {
        #[serde(rename = "inputSourceId")]
        input_source_id: String,
    },
}

impl From<&SwitcherSettings> for SettingsDto {
    fn from(settings: &SwitcherSettings) -> Self {
        Self {
            schema_version: settings.schema_version,
            enabled: settings.enabled,
            preferred_english: settings.preferred_english.as_str().to_owned(),
            preferred_japanese: settings.preferred_japanese.as_str().to_owned(),
            rules: settings.rules.iter().map(RuleDto::from).collect(),
            debug_logging: settings.debug_logging,
            snoozed_until_unix_ms: settings.snoozed_until_unix_ms,
        }
    }
}

impl TryFrom<SettingsDto> for SwitcherSettings {
    type Error = SettingsError;

    fn try_from(dto: SettingsDto) -> Result<Self, Self::Error> {
        let settings = Self {
            schema_version: dto.schema_version,
            enabled: dto.enabled,
            preferred_english: InputSourceId::parse(&dto.preferred_english)
                .map_err(invalid_settings)?,
            preferred_japanese: InputSourceId::parse(&dto.preferred_japanese)
                .map_err(invalid_settings)?,
            rules: dto
                .rules
                .into_iter()
                .map(FocusRule::try_from)
                .collect::<Result<Vec<_>, _>>()?,
            debug_logging: dto.debug_logging,
            snoozed_until_unix_ms: dto.snoozed_until_unix_ms,
        };
        settings.validate().map_err(invalid_settings)?;
        Ok(settings)
    }
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
            RuleTarget::PreferredEnglish => Self::PreferredEnglish,
            RuleTarget::PreferredJapanese => Self::PreferredJapanese,
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
            TargetDto::PreferredEnglish => Ok(Self::PreferredEnglish),
            TargetDto::PreferredJapanese => Ok(Self::PreferredJapanese),
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
        ports::settings_repository::{LoadOutcome, SettingsRepository},
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
    fn schema_v1_json_decodes() {
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
        assert_eq!(outcome.settings().preferred_english.as_str(), "english.id");
        assert_eq!(
            outcome.settings().preferred_japanese.as_str(),
            "japanese.id"
        );
        assert!(outcome.settings().debug_logging);
        assert_eq!(outcome.settings().snoozed_until_unix_ms, Some(1234));
    }

    #[test]
    fn corrupt_json_is_backed_up_and_defaults_are_returned() {
        let directory = TestDirectory::new("corrupt");
        let path = directory.join("settings.json");
        write_settings(&path, "{not-json");

        let outcome = repository(&directory).load().expect("recover");

        assert!(matches!(outcome, LoadOutcome::Recovered { .. }));
        assert_eq!(outcome.settings(), &SwitcherSettings::defaults());
        assert!(outcome.backup_path().expect("backup path").exists());
        assert!(outcome
            .warning()
            .expect("warning")
            .contains("読み込めません"));
        assert!(!path.exists());
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
