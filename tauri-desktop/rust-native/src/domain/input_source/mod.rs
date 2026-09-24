mod matcher;
mod model;

pub use matcher::RuleSet;
pub use model::{
    AxAttribute, AxElementSnapshot, FocusRule, FocusSnapshot, InputSourceId, MatchOperator,
    Platform, PlatformAppId, RuleCondition, RuleTarget, SearchScope,
};

#[cfg(test)]
mod tests;
