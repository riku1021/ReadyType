//! macOS Lab input-source smoke harness (process main thread + AppKit runloop).
//!
//! ```text
//! mise run rust-native:lab-smoke
//! ```

fn main() {
    #[cfg(not(target_os = "macos"))]
    {
        eprintln!("rust-native:lab-smoke is macOS-only");
        std::process::exit(1);
    }

    #[cfg(target_os = "macos")]
    {
        if let Err(error) = tauri_desktop_lib::run_lab_switch_smoke() {
            eprintln!("lab-smoke failed: {error}");
            std::process::exit(1);
        }
    }
}
