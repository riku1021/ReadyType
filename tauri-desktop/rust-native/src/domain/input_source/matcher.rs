use std::collections::HashSet;

use crate::domain::errors::DomainError;

use super::{
    AxElementSnapshot, FocusRule, FocusSnapshot, MatchOperator, RuleCondition, SearchScope,
};

impl RuleCondition {
    pub fn matches(&self, elements: &[AxElementSnapshot]) -> bool {
        let candidates = match self.scope() {
            SearchScope::FocusedOnly => &elements[..elements.len().min(1)],
            SearchScope::FocusedAndAncestors => elements,
        };
        let expected = self.value().to_lowercase();
        candidates.iter().any(|element| {
            let actual = element.value(self.attribute()).to_lowercase();
            match self.operator() {
                MatchOperator::Equals => actual == expected,
                MatchOperator::Contains => actual.contains(&expected),
            }
        })
    }
}

impl FocusRule {
    pub fn matches(&self, snapshot: &FocusSnapshot) -> bool {
        self.enabled()
            && self.platform() == snapshot.app().platform()
            && self.app_id() == snapshot.app().app_id()
            && self
                .conditions()
                .iter()
                .all(|condition| condition.matches(snapshot.elements()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleSet {
    rules: Vec<FocusRule>,
}

impl RuleSet {
    pub fn new(mut rules: Vec<FocusRule>) -> Result<Self, DomainError> {
        let mut ids = HashSet::with_capacity(rules.len());
        for rule in &rules {
            rule.validate()?;
            if !ids.insert(rule.id().to_owned()) {
                return Err(DomainError::DuplicateId(rule.id().to_owned()));
            }
        }
        rules.sort_by_key(FocusRule::priority);
        Ok(Self { rules })
    }

    pub fn first_match(&self, snapshot: &FocusSnapshot) -> Option<&FocusRule> {
        self.rules.iter().find(|rule| rule.matches(snapshot))
    }

    pub fn rules(&self) -> &[FocusRule] {
        &self.rules
    }
}
