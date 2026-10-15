//! macOS Lab 切替スモーク（権限付き実機）。CI / `cargo test` では実行しない。
//!
//! AppKit 窓はプロセス main + runloop が必要なため、次で起動する:
//!
//! ```text
//! mise run rust-native:lab-smoke
//! ```
//!
//! 前提: ja/en 入力ソースが利用可能。
//! **主判定は TIS `current`（メニューバー相当のソース切替）。**
//! Google 日本語などサードパーティ IME は合成 NSEvent/CGEvent では「あ」にならないことが多く、
//! 文字種の AC-2 は手動マトリクス側。欄への literal `a` は「書ける」プローブのみ。
//! 詳細: docs/input-source-switch-test-plan.md

// `cargo test`（lab-smoke feature 無し）でもモジュールは載るが、入口は bin 専用。
#![cfg_attr(not(feature = "lab-smoke"), allow(dead_code))]

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApplication, NSApplicationActivationPolicy};
use objc2_core_foundation::{kCFRunLoopDefaultMode, CFRunLoop};

use crate::application::ports::input_source::InputSourceGateway;
use crate::domain::input_source::{InputSourceId, LAB_SWITCH_FIELD_IDS};
use crate::infrastructure::platform::input_source::MacOsInputSourceGateway;

use super::{clear_field, field_string_value, focus_field, insert_literal_a, show};

fn expected_language(index: usize) -> &'static str {
    if (index + 1) % 2 == 1 {
        "ja"
    } else {
        "en"
    }
}

/// `thread::sleep` だと main の `dispatch_after`（フォーカス poll）が動かない。
fn pump_main_for(duration: Duration) {
    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        let remaining = deadline.saturating_duration_since(Instant::now());
        let slice = remaining.min(Duration::from_millis(50));
        let seconds = slice.as_secs_f64().max(0.001);
        if CFRunLoop::current().is_some() {
            let mode = unsafe { kCFRunLoopDefaultMode };
            CFRunLoop::run_in_mode(mode, seconds, true);
        } else {
            std::thread::sleep(slice);
        }
    }
}

fn settle_followup() {
    // follow-up reselect 最大 +250ms + 余裕
    pump_main_for(Duration::from_millis(400));
}

fn source_blob(source: &crate::application::ports::input_source::AvailableInputSource) -> String {
    format!(
        "{} {}",
        source.id.as_str().to_ascii_lowercase(),
        source.name.to_ascii_lowercase()
    )
}

fn is_romaji_or_ascii_ime(blob: &str) -> bool {
    ["romaji", "roman", "ascii", "英数", "alnum", "half-width"]
        .iter()
        .any(|token| blob.contains(token))
}

fn is_hiragana_preferred(blob: &str) -> bool {
    [
        "hiragana",
        "ひらがな",
        "kana",
        "japanese.kana",
        "kotoeri.japanese",
    ]
    .iter()
    .any(|token| blob.contains(token))
        && !is_romaji_or_ascii_ime(blob)
}

fn pick_sources(
    gateway: &mut MacOsInputSourceGateway<crate::infrastructure::platform::macos::SystemTisBackend>,
) -> Result<(String, String), String> {
    let sources = gateway
        .list(true, "en")
        .map_err(|error| error.to_string())?;
    let mut ja_candidates: Vec<_> = sources
        .iter()
        .filter(|source| {
            source
                .language_code
                .as_deref()
                .is_some_and(|code| code.starts_with("ja"))
        })
        .collect();
    ja_candidates.sort_by_key(|source| {
        let blob = source_blob(source);
        if is_hiragana_preferred(&blob) {
            0
        } else if is_romaji_or_ascii_ime(&blob) {
            2
        } else {
            1
        }
    });
    let ja = ja_candidates
        .first()
        .map(|source| source.id.as_str().to_owned())
        .ok_or_else(|| "no Japanese input source available".to_owned())?;
    eprintln!(
        "lab-smoke ja source={} name={:?} (glyph/IME typing is manual; this smoke checks TIS)",
        ja,
        ja_candidates.first().map(|source| source.name.as_str())
    );
    let en = sources
        .iter()
        .find(|source| {
            source
                .language_code
                .as_deref()
                .is_some_and(|code| code.starts_with("en"))
        })
        .map(|source| source.id.as_str().to_owned())
        .ok_or_else(|| "no English input source available".to_owned())?;
    Ok((ja, en))
}

/// プロセス main から呼ぶ Lab 窓スモーク（AC-2 / AC-3 相当）。
pub fn run_from_process_main() -> Result<(), String> {
    let mtm = MainThreadMarker::new().ok_or_else(|| {
        "lab-smoke must run on the process main thread (use mise run rust-native:lab-smoke)"
            .to_owned()
    })?;
    let app = NSApplication::sharedApplication(mtm);
    let _ = app.setActivationPolicy(NSApplicationActivationPolicy::Regular);
    app.finishLaunching();

    let focused = Arc::new(Mutex::new(Vec::<String>::new()));
    let focused_cb = Arc::clone(&focused);
    show(Arc::new(move |id: &str| {
        focused_cb.lock().expect("focus log").push(id.to_owned());
    }))?;

    pump_main_for(Duration::from_millis(200));

    let mut gateway = MacOsInputSourceGateway::new("en").map_err(|error| error.to_string())?;
    let (ja_id, en_id) = pick_sources(&mut gateway)?;
    let mut ready_samples = Vec::new();

    for (index, field_id) in LAB_SWITCH_FIELD_IDS.iter().enumerate() {
        clear_field(field_id)?;
        focus_field(field_id)?;
        let started = Instant::now();
        settle_followup();

        let saw_focus = focused
            .lock()
            .map_err(|_| "focus log lock poisoned".to_owned())?
            .iter()
            .any(|id| id == field_id);
        if !saw_focus {
            return Err(format!("focus callback missing for {field_id}"));
        }

        let target = if expected_language(index) == "ja" {
            ja_id.as_str()
        } else {
            en_id.as_str()
        };
        let target_id = InputSourceId::parse(target).map_err(|error| error.to_string())?;
        gateway
            .select(&target_id)
            .map_err(|error| error.to_string())?;
        settle_followup();
        let current = gateway.current().map_err(|error| error.to_string())?;
        if current.as_str() != target {
            return Err(format!(
                "TIS current after select for {field_id}: got {}, want {target}",
                current.as_str()
            ));
        }

        let lang = expected_language(index);
        // 欄が書けることの軽いプローブ（IME 変換の証明ではない）
        insert_literal_a(field_id)?;
        let value = field_string_value(field_id)?;
        if !(value.contains('a') || value.contains('A')) {
            return Err(format!(
                "field {field_id} writable probe failed; got {value:?}"
            ));
        }
        ready_samples.push(started.elapsed().as_micros());

        let manual = if lang == "ja" {
            en_id.as_str()
        } else {
            ja_id.as_str()
        };
        let manual_id = InputSourceId::parse(manual).map_err(|error| error.to_string())?;
        gateway
            .select(&manual_id)
            .map_err(|error| error.to_string())?;
        settle_followup();
        let after_manual = gateway.current().map_err(|error| error.to_string())?;
        if after_manual.as_str() != manual {
            return Err(format!(
                "manual TIS select should stick for {field_id}: got {}, want {manual}",
                after_manual.as_str()
            ));
        }
        clear_field(field_id)?;
    }

    ready_samples.sort_unstable();
    let p50 = ready_samples[ready_samples.len() / 2];
    let p95_index = (ready_samples.len() * 95 / 100).min(ready_samples.len() - 1);
    let p95 = ready_samples[p95_index];
    eprintln!(
        "lab_switch_smoke readyMicros p50={p50} p95={p95} (n={})",
        ready_samples.len()
    );
    eprintln!(
        "note: TIS select OK. 「あ」等の IME 文字種は合成キーでは Google 日本語等が応答しないため、\
         手動マトリクスで確認してください。"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::expected_language;

    #[test]
    fn lab_field_language_alternates_ja_en() {
        assert_eq!(expected_language(0), "ja");
        assert_eq!(expected_language(1), "en");
    }

    #[test]
    #[ignore = "use mise run rust-native:lab-smoke (cargo test cannot drive AppKit)"]
    fn lab_switch_smoke_focus_current_and_key() {
        panic!(
            "cargo test cannot open AppKit Lab windows (main-queue deadlock). \
             Run: mise run rust-native:lab-smoke"
        );
    }
}
