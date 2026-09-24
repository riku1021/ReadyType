use crate::domain::{
    errors::DomainError,
    input_source::{default_rules, FocusRule, InputSourceId, RuleSet},
};

pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SwitcherSettings {
    pub schema_version: u32,
    pub enabled: bool,
    pub preferred_english: InputSourceId,
    pub preferred_japanese: InputSourceId,
    pub rules: Vec<FocusRule>,
    pub debug_logging: bool,
    pub snoozed_until_unix_ms: Option<u64>,
}

impl SwitcherSettings {
    pub fn defaults() -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            enabled: true,
            preferred_english: InputSourceId::parse("com.apple.keylayout.ABC")
                .expect("default English input source must be valid"),
            preferred_japanese: InputSourceId::parse("com.google.inputmethod.Japanese.base")
                .expect("default Japanese input source must be valid"),
            rules: default_rules(),
            debug_logging: false,
            snoozed_until_unix_ms: None,
        }
    }

    pub fn validate(&self) -> Result<(), DomainError> {
        if self.schema_version != CURRENT_SCHEMA_VERSION {
            return Err(DomainError::InvalidInput(format!(
                "unsupported settings schema version: {}",
                self.schema_version
            )));
        }
        RuleSet::new(self.rules.clone())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SwitcherSettings, CURRENT_SCHEMA_VERSION};

    #[test]
    fn defaults_include_terminal_cursor_and_vscode() {
        let settings = SwitcherSettings::defaults();

        assert_eq!(settings.schema_version, CURRENT_SCHEMA_VERSION);
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.apple.Terminal"));
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.todesktop.230313mzl4w4u92"));
        assert!(settings
            .rules
            .iter()
            .any(|rule| rule.app_id() == "com.microsoft.VSCode"));
    }

    #[test]
    fn defaults_use_expected_input_sources_and_enabled_state() {
        let settings = SwitcherSettings::defaults();

        assert!(settings.enabled);
        assert_eq!(
            settings.preferred_english.as_str(),
            "com.apple.keylayout.ABC"
        );
        assert_eq!(
            settings.preferred_japanese.as_str(),
            "com.google.inputmethod.Japanese.base"
        );
        assert!(!settings.debug_logging);
        assert_eq!(settings.snoozed_until_unix_ms, None);
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_a_schema_version_newer_than_supported() {
        let mut settings = SwitcherSettings::defaults();
        settings.schema_version = CURRENT_SCHEMA_VERSION + 1;

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_rule_ids() {
        let mut settings = SwitcherSettings::defaults();
        settings.rules.push(settings.rules[0].clone());

        assert!(settings.validate().is_err());
    }
}
