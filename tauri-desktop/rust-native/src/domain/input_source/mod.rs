mod matcher;
mod model;
mod presets;

pub use matcher::RuleSet;
pub use model::{
    canonicalize_language_codes_by_family, is_ime_latin_mode, AxAttribute, AxElementSnapshot,
    FocusRule, FocusSnapshot, InputSourceId, LanguageTag, MatchOperator, Platform, PlatformAppId,
    RuleCondition, RuleTarget, SearchScope,
};
pub use presets::default_rules;

#[cfg(test)]
mod tests;
