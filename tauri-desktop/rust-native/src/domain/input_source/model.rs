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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FocusSnapshot {
    app: PlatformAppId,
    generation: u64,
    elements: Vec<AxElementSnapshot>,
}

impl FocusSnapshot {
    pub fn new(app: PlatformAppId, generation: u64, elements: Vec<AxElementSnapshot>) -> Self {
        Self {
            app,
            generation,
            elements,
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
