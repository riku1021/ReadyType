//! Propose rule conditions and display labels from a focus snapshot.

use super::{
    AxAttribute, AxElementSnapshot, FocusSnapshot, MatchOperator, RuleCondition, SearchScope,
};

const MAX_CONDITIONS: usize = 3;
const GENERIC_LABELS: &[&str] = &[
    "text",
    "textarea",
    "text field",
    "textfield",
    "edit",
    "input",
    "field",
    "axtextarea",
    "axtextfield",
    "axtextareanonsensitive",
];

/// Build matching conditions for a new field rule from the focused element chain.
pub fn propose_conditions(snapshot: &FocusSnapshot) -> Vec<RuleCondition> {
    let Some(focused) = snapshot.elements().first() else {
        return Vec::new();
    };

    let mut conditions = Vec::with_capacity(MAX_CONDITIONS);

    if let Some(condition) =
        condition_from(focused, AxAttribute::DomIdentifier, MatchOperator::Contains)
    {
        conditions.push(condition);
    } else if let Some(condition) =
        condition_from(focused, AxAttribute::Identifier, MatchOperator::Equals)
    {
        conditions.push(condition);
    } else if let Some(token) = distinctive_class_token(focused.value(AxAttribute::DomClassList)) {
        if let Ok(condition) = RuleCondition::new(
            AxAttribute::DomClassList,
            MatchOperator::Contains,
            SearchScope::FocusedOnly,
            &token,
        ) {
            conditions.push(condition);
        }
    }

    if let Some(description) = meaningful_text(focused.value(AxAttribute::Description)) {
        if let Ok(condition) = RuleCondition::new(
            AxAttribute::Description,
            MatchOperator::Contains,
            SearchScope::FocusedOnly,
            description,
        ) {
            push_unique(&mut conditions, condition);
        }
    } else if let Some(title) = meaningful_text(focused.value(AxAttribute::Title)) {
        if let Ok(condition) = RuleCondition::new(
            AxAttribute::Title,
            MatchOperator::Contains,
            SearchScope::FocusedOnly,
            title,
        ) {
            push_unique(&mut conditions, condition);
        }
    }

    let has_strong_id = conditions.iter().any(|condition| {
        matches!(
            condition.attribute(),
            AxAttribute::DomIdentifier | AxAttribute::Identifier
        )
    });

    if conditions.is_empty() {
        if let Some(condition) = condition_from(focused, AxAttribute::Role, MatchOperator::Equals) {
            conditions.push(condition);
        }
    } else if !has_strong_id && conditions.len() < MAX_CONDITIONS {
        if let Some(role) = trimmed(focused.value(AxAttribute::Role)) {
            if role.to_ascii_lowercase().contains("text") {
                if let Ok(condition) = RuleCondition::new(
                    AxAttribute::Role,
                    MatchOperator::Contains,
                    SearchScope::FocusedOnly,
                    "text",
                ) {
                    push_unique(&mut conditions, condition);
                }
            }
        }
    }

    conditions.truncate(MAX_CONDITIONS);
    conditions
}

/// Field label from AX text when available. UI locale fallback is the caller's job.
///
/// Order: Accessible Name (Description / Title) → Placeholder → RoleDescription →
/// DomIdentifier tail. TitleUIElement resolution is intentionally out of scope.
pub fn propose_field_label(snapshot: &FocusSnapshot) -> Option<String> {
    let focused = snapshot.elements().first()?;

    for attribute in [
        AxAttribute::Description,
        AxAttribute::Title,
        AxAttribute::Placeholder,
        AxAttribute::RoleDescription,
    ] {
        if let Some(text) = meaningful_text(focused.value(attribute)) {
            return Some(shorten(text, 48));
        }
    }

    if let Some(id) = trimmed(focused.value(AxAttribute::DomIdentifier)) {
        if let Some(tail) = id.rsplit(['.', '_', '-']).next() {
            if meaningful_text(tail).is_some() {
                return Some(shorten(tail, 48));
            }
        }
    }

    None
}

/// Stable rule name for persistence (`AI Chat`, or `"field"` when AX has no label).
/// Not localized — UI copy belongs in the frontend / tray packs.
pub fn propose_rule_name(snapshot: &FocusSnapshot) -> String {
    propose_field_label(snapshot).unwrap_or_else(|| RULE_NAME_FIELD_FALLBACK.to_owned())
}

/// Non-localized slug used when AX does not yield a display label.
pub const RULE_NAME_FIELD_FALLBACK: &str = "field";

fn condition_from(
    element: &AxElementSnapshot,
    attribute: AxAttribute,
    operator: MatchOperator,
) -> Option<RuleCondition> {
    let value = trimmed(element.value(attribute))?;
    RuleCondition::new(attribute, operator, SearchScope::FocusedOnly, value).ok()
}

fn push_unique(conditions: &mut Vec<RuleCondition>, candidate: RuleCondition) {
    let duplicate = conditions.iter().any(|existing| {
        existing.attribute() == candidate.attribute()
            && existing.operator() == candidate.operator()
            && existing.scope() == candidate.scope()
            && existing.value().eq_ignore_ascii_case(candidate.value())
    });
    if !duplicate {
        conditions.push(candidate);
    }
}

fn trimmed(value: &str) -> Option<&str> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

fn meaningful_text(value: &str) -> Option<&str> {
    let trimmed = trimmed(value)?;
    let lower = trimmed.to_ascii_lowercase();
    if trimmed.chars().count() < 2 {
        return None;
    }
    if GENERIC_LABELS.iter().any(|label| *label == lower) {
        return None;
    }
    Some(trimmed)
}

fn distinctive_class_token(class_list: &str) -> Option<String> {
    let mut best: Option<String> = None;
    for token in class_list.split_whitespace() {
        let lower = token.to_ascii_lowercase();
        if token.len() < 4 {
            continue;
        }
        if GENERIC_LABELS.iter().any(|label| lower.contains(label)) && token.len() < 12 {
            continue;
        }
        if (lower.contains("input")
            || lower.contains("composer")
            || lower.contains("editor")
            || lower.contains("chat")
            || lower.contains("textarea")
            || token.contains('-')
            || token.contains('_'))
            && best
                .as_ref()
                .is_none_or(|current| token.len() > current.len())
        {
            best = Some(token.to_owned());
        }
    }
    best
}

fn shorten(value: &str, max_chars: usize) -> String {
    let count = value.chars().count();
    if count <= max_chars {
        return value.to_owned();
    }
    let truncated: String = value.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests;
