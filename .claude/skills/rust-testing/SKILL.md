---
name: rust-testing
description: >-
  ReadyType Rust の cargo test / テーブル駆動 / port ダブル / Clippy 方針。
  rust-native のテスト追加・修正時に使う。
---

# Rust Testing Skill

正本ルール: `.claude/rules/rust-testing.md`。

## 優先順位

1. domain / use case の純粋ロジック（**ソース隣接** `#[cfg(test)]`）
2. port は Fake / ダブルで差し替え
3. Tauri コマンドは薄いので最小限
4. UI / IPC の見た目側は Vitest（フロント）
5. FE の `tests/` ミラーにユニットを寄せない。クレート `tests/` は公開 API 統合のみ

## コマンド

```bash
cd tauri-desktop/rust-native
cargo test
cargo clippy --all-targets -- -D warnings
```

## テンプレ（隣接テスト）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_empty() {
        assert!(GuestName::parse("").is_err());
    }

    #[test]
    fn parse_accepts_trimmed() {
        let name = GuestName::parse("  A  ").expect("ok");
        assert_eq!(name.as_str(), "A");
    }
}
```

## テーブル駆動

```rust
#[test]
fn cases() {
    let cases: &[(&str, bool)] = &[("Alice", true), ("", false), ("  ", false)];
    for &(input, ok) in cases {
        assert_eq!(GuestName::parse(input).is_ok(), ok, "input={input:?}");
    }
}
```

## Port Fake

```rust
struct FakePlatform { fail: bool }

impl PlatformGateway for FakePlatform {
    fn sync_window_chrome(&self) -> Result<(), PlatformError> {
        if self.fail { Err(PlatformError::Unavailable) } else { Ok(()) }
    }
}
```

## Done 条件

- [ ] 成功・代表的失敗をカバー
- [ ] `cargo test` 成功
- [ ] Clippy `-D warnings` 成功
- [ ] フレイキーな実 OS 依存を増やしていない
