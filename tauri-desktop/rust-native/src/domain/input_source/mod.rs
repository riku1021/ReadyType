mod matcher;
mod model;
mod presets;
mod propose;

pub use matcher::RuleSet;
pub use model::{
    canonicalize_language_codes_by_family, is_ime_latin_mode, AxAttribute, AxElementSnapshot,
    FocusRule, FocusSnapshot, InputSourceId, LanguageTag, MatchOperator, Platform, PlatformAppId,
    RuleCondition, RuleTarget, SearchScope,
};
pub use presets::{
    default_rules, readytype_lab_switch_rules, LAB_SWITCH_FIELD_IDS, READYTYPE_BUNDLE_ID,
};
pub use propose::{propose_conditions, propose_field_label, propose_rule_name};

#[cfg(test)]
mod tests;
