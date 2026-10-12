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
fn missing_onboarding_completed_defaults_to_true_for_existing_files() {
    let directory = TestDirectory::new("onboarding-missing");
    write_settings(
        &directory.join("settings.json"),
        r#"{
            "schemaVersion": 1,
            "enabled": true,
            "languagePreferences": {},
            "rules": [],
            "debugLogging": false,
            "snoozedUntilUnixMs": null
        }"#,
    );

    let outcome = repository(&directory).load().expect("decode");

    assert!(outcome.settings().onboarding_completed);
}

#[test]
fn onboarding_completed_false_round_trips() {
    let directory = TestDirectory::new("onboarding-false");
    let repository = repository(&directory);
    let expected = SwitcherSettings::defaults();
    assert!(!expected.onboarding_completed);

    repository.save(&expected).expect("save");
    let outcome = repository.load().expect("load");

    assert!(!outcome.settings().onboarding_completed);
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
