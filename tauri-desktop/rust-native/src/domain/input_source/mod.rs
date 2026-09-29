mod matcher;
mod model;
mod presets;

pub use matcher::RuleSet;
pub use model::{
    AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot, InputSourceId, LanguageTag,
    MatchOperator, Platform, PlatformAppId, RuleCondition, RuleTarget, SearchScope,
};
pub use presets::default_rules;

#[cfg(test)]
mod tests;
