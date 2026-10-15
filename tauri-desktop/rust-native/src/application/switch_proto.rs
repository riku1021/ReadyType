//! Opt-in prototype flags for latency.md candidates (default OFF).
//!
//! See docs/input-source-switch-test-plan.md. Unset env = current production behavior.

/// `READYTYPE_PROTO_CURRENT_CACHE=1` — CurrentInputSourceCache TTL + warm.
pub const ENV_CURRENT_CACHE: &str = "READYTYPE_PROTO_CURRENT_CACHE";

/// `READYTYPE_PROTO_TIS_NOTIFY=1` — invalidate cache on TIS change notification.
pub const ENV_TIS_NOTIFY: &str = "READYTYPE_PROTO_TIS_NOTIFY";

/// `READYTYPE_PROTO_MANUAL_OVERRIDE=1` — focus-key manual override latch.
pub const ENV_MANUAL_OVERRIDE: &str = "READYTYPE_PROTO_MANUAL_OVERRIDE";

/// `READYTYPE_PROTO_DRAIN_FOCUS=1` — drain queued Focus, keep latest only.
pub const ENV_DRAIN_FOCUS: &str = "READYTYPE_PROTO_DRAIN_FOCUS";

/// `READYTYPE_PROTO_PUBLISH_FINGERPRINT=1` — skip publish when UI-relevant state unchanged.
pub const ENV_PUBLISH_FINGERPRINT: &str = "READYTYPE_PROTO_PUBLISH_FINGERPRINT";

/// Reserved (Wave 2): AX fast path.
pub const ENV_AX_FAST: &str = "READYTYPE_PROTO_AX_FAST";

/// Reserved (Wave 3): tray diff / latest-wins.
pub const ENV_TRAY_DIFF: &str = "READYTYPE_PROTO_TRAY_DIFF";

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

pub fn current_cache() -> bool {
    flag_enabled(ENV_CURRENT_CACHE)
}

pub fn tis_notify() -> bool {
    flag_enabled(ENV_TIS_NOTIFY)
}

pub fn manual_override() -> bool {
    flag_enabled(ENV_MANUAL_OVERRIDE)
}

pub fn drain_focus() -> bool {
    flag_enabled(ENV_DRAIN_FOCUS)
}

pub fn publish_fingerprint() -> bool {
    flag_enabled(ENV_PUBLISH_FINGERPRINT)
}

pub fn ax_fast() -> bool {
    flag_enabled(ENV_AX_FAST)
}

pub fn tray_diff() -> bool {
    flag_enabled(ENV_TRAY_DIFF)
}

/// Currently enabled prototype flags (empty = all PROTO off = current production).
pub fn active_slugs() -> Vec<&'static str> {
    let mut out = Vec::new();
    if current_cache() {
        out.push("currentCache");
    }
    if tis_notify() {
        out.push("tisNotify");
    }
    if manual_override() {
        out.push("manualOverride");
    }
    if drain_focus() {
        out.push("drainFocus");
    }
    if publish_fingerprint() {
        out.push("publishFingerprint");
    }
    if ax_fast() {
        out.push("axFast");
    }
    if tray_diff() {
        out.push("trayDiff");
    }
    out
}

/// Serialize tests that mutate PROTO / ablation env vars.
#[cfg(test)]
pub static PROTO_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub fn clear_all_proto_env() {
    // SAFETY: caller holds PROTO_ENV_LOCK (or accepts process-global risk).
    unsafe {
        std::env::remove_var(ENV_CURRENT_CACHE);
        std::env::remove_var(ENV_TIS_NOTIFY);
        std::env::remove_var(ENV_MANUAL_OVERRIDE);
        std::env::remove_var(ENV_DRAIN_FOCUS);
        std::env::remove_var(ENV_PUBLISH_FINGERPRINT);
        std::env::remove_var(ENV_AX_FAST);
        std::env::remove_var(ENV_TRAY_DIFF);
    }
}

#[cfg(test)]
pub fn with_clean_proto_env<R>(f: impl FnOnce() -> R) -> R {
    let _guard = PROTO_ENV_LOCK.lock().expect("proto env lock");
    clear_all_proto_env();
    f()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flags_default_off_when_unset() {
        assert!(!flag_enabled("READYTYPE_PROTO_UNSET_FOR_TEST_ONLY"));
    }
}
