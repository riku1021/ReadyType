use serde::{Deserialize, Serialize};

use crate::{
    application::{
        input_source_switcher::SwitchDecision,
        ports::{focus_monitor::AccessibilityPermission, input_source::AvailableInputSource},
        settings::SwitcherSettings,
        switcher_state::SwitcherState,
    },
    domain::input_source::{
        AxAttribute, FocusRule, InputSourceId, LanguageTag, MatchOperator, Platform, RuleCondition,
        RuleTarget, SearchScope,
    },
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateDto {
    pub version: u64,
    pub rules_version: u64,
    pub enabled: bool,
    pub snoozed_until_unix_ms: Option<u64>,
    pub current_app: Option<AppDto>,
    pub matched_rule_id: Option<String>,
    pub current_input_source_id: Option<String>,
    pub target_input_source_id: Option<String>,
    pub last_decision: Option<DecisionDto>,
}

impl From<&SwitcherState> for StateDto {
    fn from(state: &SwitcherState) -> Self {
        Self {
            version: state.version,
            rules_version: state.rules_version,
            enabled: state.enabled,
            snoozed_until_unix_ms: state.snoozed_until_unix_ms,
            current_app: state.current_app.as_ref().map(AppDto::from),
            matched_rule_id: state.matched_rule_id.clone(),
            current_input_source_id: state
                .current_input_source
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            target_input_source_id: state
                .target_input_source
                .as_ref()
                .map(|id| id.as_str().to_owned()),
            last_decision: state.last_decision.map(DecisionDto::from),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StateEventDto {
    pub version: u64,
    pub state: StateDto,
}

impl From<&SwitcherState> for StateEventDto {
    fn from(state: &SwitcherState) -> Self {
        Self {
            version: state.version,
            state: StateDto::from(state),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDto {
    pub platform: PlatformDto,
    pub app_id: String,
    pub pid: i32,
}

impl From<&crate::domain::input_source::PlatformAppId> for AppDto {
    fn from(app: &crate::domain::input_source::PlatformAppId) -> Self {
        Self {
            platform: PlatformDto::from(app.platform()),
            app_id: app.app_id().to_owned(),
            pid: app.pid(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PlatformDto {
    MacOs,
    Windows,
    Linux,
}

impl From<Platform> for PlatformDto {
    fn from(platform: Platform) -> Self {
        match platform {
            Platform::MacOs => Self::MacOs,
            Platform::Windows => Self::Windows,
            Platform::Linux => Self::Linux,
        }
    }
}

impl From<PlatformDto> for Platform {
    fn from(platform: PlatformDto) -> Self {
        match platform {
            PlatformDto::MacOs => Self::MacOs,
            PlatformDto::Windows => Self::Windows,
            PlatformDto::Linux => Self::Linux,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DecisionDto {
    Ignored,
    Disabled,
    Snoozed,
    NoRule,
    AlreadySelected,
    Switched,
    Failed,
}

impl From<SwitchDecision> for DecisionDto {
    fn from(decision: SwitchDecision) -> Self {
        match decision {
            SwitchDecision::Ignored => Self::Ignored,
            SwitchDecision::Disabled => Self::Disabled,
            SwitchDecision::Snoozed => Self::Snoozed,
            SwitchDecision::NoRule => Self::NoRule,
            SwitchDecision::AlreadySelected => Self::AlreadySelected,
            SwitchDecision::Switched => Self::Switched,
            SwitchDecision::Failed => Self::Failed,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionDto {
    Granted,
    Denied,
    Unsupported,
}

impl From<AccessibilityPermission> for PermissionDto {
    fn from(permission: AccessibilityPermission) -> Self {
        match permission {
            AccessibilityPermission::Granted => Self::Granted,
            AccessibilityPermission::Denied => Self::Denied,
            AccessibilityPermission::Unsupported => Self::Unsupported,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InputSourceDto {
    pub id: String,
    pub name: String,
    pub language_code: Option<String>,
    pub available: bool,
}

impl From<AvailableInputSource> for InputSourceDto {
    fn from(source: AvailableInputSource) -> Self {
        Self {
            id: source.id.as_str().to_owned(),
            name: source.name,
            language_code: source.language_code,
            available: source.available,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleDto {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub platform: PlatformDto,
    pub app_id: String,
    pub priority: u32,
    pub conditions: Vec<ConditionDto>,
    pub target: RuleTargetDto,
}

impl TryFrom<RuleDto> for FocusRule {
    type Error = String;

    fn try_from(rule: RuleDto) -> Result<Self, Self::Error> {
        let conditions = rule
            .conditions
            .into_iter()
            .map(RuleCondition::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        FocusRule::new(
            &rule.id,
            &rule.name,
            rule.enabled,
            rule.platform.into(),
            &rule.app_id,
            rule.priority,
            conditions,
            rule.target.try_into()?,
        )
        .map_err(|error| error.to_string())
    }
}

impl From<&FocusRule> for RuleDto {
    fn from(rule: &FocusRule) -> Self {
        Self {
            id: rule.id().to_owned(),
            name: rule.name().to_owned(),
            enabled: rule.enabled(),
            platform: rule.platform().into(),
            app_id: rule.app_id().to_owned(),
            priority: rule.priority(),
            conditions: rule.conditions().iter().map(ConditionDto::from).collect(),
            target: RuleTargetDto::from(rule.target()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConditionDto {
    pub attribute: AxAttributeDto,
    pub operator: MatchOperatorDto,
    pub scope: SearchScopeDto,
    pub value: String,
}

impl TryFrom<ConditionDto> for RuleCondition {
    type Error = String;

    fn try_from(condition: ConditionDto) -> Result<Self, Self::Error> {
        RuleCondition::new(
            condition.attribute.into(),
            condition.operator.into(),
            condition.scope.into(),
            &condition.value,
        )
        .map_err(|error| error.to_string())
    }
}

impl From<&RuleCondition> for ConditionDto {
    fn from(condition: &RuleCondition) -> Self {
        Self {
            attribute: condition.attribute().into(),
            operator: condition.operator().into(),
            scope: condition.scope().into(),
            value: condition.value().to_owned(),
        }
    }
}

macro_rules! wire_enum {
    ($wire:ident, $domain:ty, { $($variant:ident),+ $(,)? }) => {
        #[derive(Debug, Clone, Copy, Serialize, Deserialize)]
        #[serde(rename_all = "camelCase")]
        pub enum $wire { $($variant),+ }

        impl From<$wire> for $domain {
            fn from(value: $wire) -> Self {
                match value { $($wire::$variant => Self::$variant),+ }
            }
        }

        impl From<$domain> for $wire {
            fn from(value: $domain) -> Self {
                match value { $(<$domain>::$variant => Self::$variant),+ }
            }
        }
    };
}

wire_enum!(AxAttributeDto, AxAttribute, {
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
});
wire_enum!(MatchOperatorDto, MatchOperator, { Equals, Contains });
wire_enum!(SearchScopeDto, SearchScope, {
    FocusedOnly,
    FocusedAndAncestors,
});

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuleTargetDto {
    PreferredLanguage {
        language: String,
    },
    /// Legacy wire variants accepted on input only.
    PreferredEnglish,
    PreferredJapanese,
    Exact {
        #[serde(rename = "inputSourceId")]
        input_source_id: String,
    },
}

impl TryFrom<RuleTargetDto> for RuleTarget {
    type Error = String;

    fn try_from(target: RuleTargetDto) -> Result<Self, Self::Error> {
        match target {
            RuleTargetDto::PreferredLanguage { language } => LanguageTag::parse(&language)
                .map(Self::PreferredLanguage)
                .map_err(|error| error.to_string()),
            RuleTargetDto::PreferredEnglish => Ok(Self::PreferredLanguage(LanguageTag::en())),
            RuleTargetDto::PreferredJapanese => Ok(Self::PreferredLanguage(LanguageTag::ja())),
            RuleTargetDto::Exact { input_source_id } => InputSourceId::parse(&input_source_id)
                .map(Self::Exact)
                .map_err(|error| error.to_string()),
        }
    }
}

impl From<&RuleTarget> for RuleTargetDto {
    fn from(target: &RuleTarget) -> Self {
        match target {
            RuleTarget::PreferredLanguage(tag) => Self::PreferredLanguage {
                language: tag.as_str().to_owned(),
            },
            RuleTarget::Exact(id) => Self::Exact {
                input_source_id: id.as_str().to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsDto {
    pub language_preferences: std::collections::BTreeMap<String, Option<String>>,
    pub debug_logging: bool,
    /// Preference: `system` | packs from shared i18n registry.
    pub ui_locale: String,
    /// Resolved display language from shared i18n registry.
    pub resolved_ui_locale: String,
    /// Quick Set グローバルショートカット（Tauri accelerator）。
    pub quick_set_shortcut: String,
    /// 起動時登録などを含め、現在 OS に登録されているか。
    pub quick_set_shortcut_registered: bool,
    /// 初回オンボーディング完了済みか。
    pub onboarding_completed: bool,
    /// 入力ソース id ごとの最終既知 LocalizedName（OS 削除後も UI 表示用に保持）。
    pub input_source_display_names: std::collections::BTreeMap<String, String>,
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
            language_preferences,
            debug_logging: settings.debug_logging,
            ui_locale: settings.ui_locale.clone(),
            resolved_ui_locale: SwitcherSettings::resolve_ui_locale(&settings.ui_locale),
            quick_set_shortcut: settings.quick_set_shortcut.clone(),
            // AppHandle なしでは不明。コマンド側で上書きする。
            quick_set_shortcut_registered: false,
            onboarding_completed: settings.onboarding_completed,
            input_source_display_names: settings.input_source_display_names.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct QuickSetSessionDto {
    pub app_id: String,
    pub app_display_name: String,
    pub platform: PlatformDto,
    pub pid: i32,
    pub proposed_name: String,
    pub proposed_conditions: Vec<ConditionDto>,
    pub matched_rule_id: Option<String>,
    pub matched_target: Option<RuleTargetDto>,
    pub sources: Vec<InputSourceDto>,
    pub language_preferences: std::collections::BTreeMap<String, Option<String>>,
}

impl From<&crate::application::quick_set::QuickSetSession> for QuickSetSessionDto {
    fn from(session: &crate::application::quick_set::QuickSetSession) -> Self {
        Self {
            app_id: session.app_id.clone(),
            app_display_name: session.app_display_name.clone(),
            platform: PlatformDto::from(session.platform),
            pid: session.pid,
            proposed_name: session.proposed_name.clone(),
            proposed_conditions: session
                .proposed_conditions
                .iter()
                .map(ConditionDto::from)
                .collect(),
            matched_rule_id: session.matched_rule_id.clone(),
            matched_target: session.matched_target.as_ref().map(RuleTargetDto::from),
            sources: session
                .sources
                .iter()
                .cloned()
                .map(InputSourceDto::from)
                .collect(),
            language_preferences: session
                .language_preferences
                .iter()
                .map(|(tag, preferred)| {
                    (
                        tag.as_str().to_owned(),
                        preferred.as_ref().map(|id| id.as_str().to_owned()),
                    )
                })
                .collect(),
        }
    }
}

/// Hotkey open payload: always show the window; session may be unavailable.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum QuickSetOpenEventDto {
    #[serde(rename_all = "camelCase")]
    Ready {
        session: Box<QuickSetSessionDto>,
    },
    Unavailable,
}

#[cfg(test)]
mod tests;
