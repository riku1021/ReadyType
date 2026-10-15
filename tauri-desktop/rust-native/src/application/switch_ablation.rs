//! Env flags for validating stability/speed mitigations (see docs/input-source-switch-test-plan.md).
//!
//! Infrastructure (TIS follow-up) reads the same env names without importing this module,
//! to keep the onion dependency direction.

/// `READYTYPE_ABLATION_NO_FOLLOWUP=1` — skip post-select follow-up reselects (+40/120/250ms).
pub const ENV_NO_FOLLOWUP: &str = "READYTYPE_ABLATION_NO_FOLLOWUP";

/// `READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT=1` — skip `select` when current already matches target.
pub const ENV_NO_SAME_SOURCE_RESELECT: &str = "READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT";

/// `READYTYPE_ABLATION_NO_RETRY=1` — skip list+retry after the first `select` failure.
pub const ENV_NO_RETRY: &str = "READYTYPE_ABLATION_NO_RETRY";

fn flag_enabled(name: &str) -> bool {
    match std::env::var(name) {
        Ok(value) => {
            let trimmed = value.trim();
            trimmed == "1"
                || trimmed.eq_ignore_ascii_case("true")
                || trimmed.eq_ignore_ascii_case("yes")
        }
        Err(_) => false,
    }
}

pub fn no_followup() -> bool {
    flag_enabled(ENV_NO_FOLLOWUP)
}

pub fn no_same_source_reselect() -> bool {
    flag_enabled(ENV_NO_SAME_SOURCE_RESELECT)
}

pub fn no_retry() -> bool {
    flag_enabled(ENV_NO_RETRY)
}

/// Currently enabled ablation flags (empty = production defaults).
pub fn active_slugs() -> Vec<&'static str> {
    let mut out = Vec::new();
    if no_followup() {
        out.push("noFollowup");
    }
    if no_same_source_reselect() {
        out.push("noSameSourceReselect");
    }
    if no_retry() {
        out.push("noRetry");
    }
    out
}

/// Serialize tests that mutate ablation env vars.
#[cfg(test)]
pub static ABLATION_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_default_off() {
        // Do not assert global env emptiness (parallel tests); only the parser.
        assert!(!flag_enabled("READYTYPE_ABLATION_UNSET_FOR_TEST_ONLY"));
        assert!(flag_enabled_value("1"));
        assert!(flag_enabled_value("true"));
        assert!(flag_enabled_value("YES"));
        assert!(!flag_enabled_value("0"));
        assert!(!flag_enabled_value("no"));
    }

    fn flag_enabled_value(value: &str) -> bool {
        let trimmed = value.trim();
        trimmed == "1"
            || trimmed.eq_ignore_ascii_case("true")
            || trimmed.eq_ignore_ascii_case("yes")
    }
}
