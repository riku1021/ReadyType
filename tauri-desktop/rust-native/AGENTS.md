# AGENTS.md — Rust native (tauri-desktop/rust-native)

このディレクトリでは Tauri / Onion の Rust を扱う。詳細はリポジトリ直下の `AGENTS.md` と `.claude/rules/`。

## 優先スキル

- `rust-backend`
- `rust-testing`
- `agent-behavior`

## ローカル制約

- 依存は外側 → 内側のみ（`domain` / `application` に `tauri`・OS を持ち込まない）
- コマンドは薄く、処理は use case
- 新規コマンドは `generate_handler!` とフロント `invoke` 名を一致させる
- 完了前: `cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test`
