use std::{fmt, path::Path};

use crate::application::settings::SwitcherSettings;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    Io(String),
    Serialization(String),
    InvalidSettings(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "settings I/O error: {message}"),
            Self::Serialization(message) => {
                write!(formatter, "settings serialization error: {message}")
            }
            Self::InvalidSettings(message) => write!(formatter, "invalid settings: {message}"),
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
            | Self::Recovered { settings, .. } => settings,
        }
    }

    pub fn backup_path(&self) -> Option<&Path> {
        match self {
            Self::Recovered { backup_path, .. } => Some(backup_path),
            Self::Loaded { .. } | Self::Defaults { .. } => None,
        }
    }

    pub fn warning(&self) -> Option<&str> {
        match self {
            Self::Recovered { warning, .. } => Some(warning),
            Self::Loaded { .. } | Self::Defaults { .. } => None,
        }
    }
}

pub trait SettingsRepository: Send + Sync {
    fn load(&self) -> Result<LoadOutcome, SettingsError>;
    fn save(&self, settings: &SwitcherSettings) -> Result<(), SettingsError>;
}
