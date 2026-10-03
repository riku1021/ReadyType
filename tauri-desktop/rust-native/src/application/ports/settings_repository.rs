use std::sync::Arc;
use std::{fmt, path::Path};

use crate::application::settings::SwitcherSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    Io(String),
    Serialization(String),
    InvalidSettings(String),
    /// Corrupt load fell back to defaults; persist is blocked until the app is reset intentionally.
    WritesSuspended,
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "settings I/O error: {message}"),
            Self::Serialization(message) => {
                write!(formatter, "settings serialization error: {message}")
            }
            Self::InvalidSettings(message) => write!(formatter, "invalid settings: {message}"),
            Self::WritesSuspended => write!(
                formatter,
                "settings writes suspended after unrecovered corruption"
            ),
        }
    }
}

impl std::error::Error for SettingsError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadOutcome {
    Loaded {
        settings: SwitcherSettings,
    },
    Defaults {
        settings: SwitcherSettings,
    },
    /// Main file was unreadable; last-known-good `.bak` was restored to main.
    RestoredFromBackup {
        settings: SwitcherSettings,
        corrupt_path: std::path::PathBuf,
        warning: String,
    },
    /// Main (and `.bak` if any) were unreadable; in-memory defaults only. Writes are suspended.
    Recovered {
        settings: SwitcherSettings,
        backup_path: std::path::PathBuf,
        warning: String,
    },
}

impl LoadOutcome {
    pub fn settings(&self) -> &SwitcherSettings {
        match self {
            Self::Loaded { settings }
            | Self::Defaults { settings }
            | Self::RestoredFromBackup { settings, .. }
            | Self::Recovered { settings, .. } => settings,
        }
    }

    pub fn backup_path(&self) -> Option<&Path> {
        match self {
            Self::Recovered { backup_path, .. } => Some(backup_path),
            Self::RestoredFromBackup { corrupt_path, .. } => Some(corrupt_path),
            Self::Loaded { .. } | Self::Defaults { .. } => None,
        }
    }

    pub fn warning(&self) -> Option<&str> {
        match self {
            Self::Recovered { warning, .. } | Self::RestoredFromBackup { warning, .. } => {
                Some(warning)
            }
            Self::Loaded { .. } | Self::Defaults { .. } => None,
        }
    }

    /// True when disk must not be overwritten with in-memory defaults.
    pub fn writes_suspended(&self) -> bool {
        matches!(self, Self::Recovered { .. })
    }
}

pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> Result<LoadOutcome, SettingsError>;
    fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError>;
}

impl<T: SettingsRepository + ?Sized> SettingsRepository for Arc<T> {
    fn load(&self) -> Result<LoadOutcome, SettingsError> {
        (**self).load()
    }

    fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError> {
        (**self).save(settings)
    }
}
