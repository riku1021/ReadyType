use std::fs;
use std::path::PathBuf;

fn main() {
    ensure_tis_list_sidecar_placeholder();
    tauri_build::build();
}

/// macOS `bundle.externalBin` は compile 前にファイル存在が必須。
/// 実体は `scripts/macos/prepare-tis-list-sidecar.sh` が上書きする。
/// 配置は `target/sidecar/`（ソース監視外。`binaries/` だと再ビルドループになる）。
fn ensure_tis_list_sidecar_placeholder() {
    let Ok(triple) = std::env::var("TARGET") else {
        return;
    };
    if !triple.contains("apple-darwin") {
        return;
    }
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into()));
    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("target"));
    let dest = target_dir
        .join("sidecar")
        .join(format!("tis-list-enabled-{triple}"));
    if dest.is_file() {
        return;
    }
    if let Some(parent) = dest.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            println!(
                "cargo:warning=failed to create {}: {error}",
                parent.display()
            );
            return;
        }
    }
    let script = "#!/bin/sh\necho \"tis-list-enabled placeholder\" >&2\nexit 1\n";
    if let Err(error) = fs::write(&dest, script) {
        println!("cargo:warning=failed to write {}: {error}", dest.display());
        return;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(meta) = fs::metadata(&dest) {
            let mut perms = meta.permissions();
            perms.set_mode(0o755);
            let _ = fs::set_permissions(&dest, perms);
        }
    }
}
