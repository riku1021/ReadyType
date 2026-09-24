---
description: Rust ネイティブ層のテスト方針（ユニット・テーブル駆動・port ダブル・コマンド）
globs:
  - "tauri-desktop/rust-native/**/*.rs"
alwaysApply: false
---

# Rust Testing Rules

## Testing Stack

| 項目 | 内容 |
|------|------|
| Runner | `cargo test` |
| 配置 | モジュール隣接 `#[cfg(test)] mod tests`、またはクレート `tests/` |
| Mock | 手書きテストダブル、または必要なら mockall 等（追加は承認必須） |

フロントの Vitest（`mockIPC`）と責務を分ける。Rust は domain / use case / infrastructure を、IPC 契約の UI 側は Vitest を担当する。

## 方針

1. **domain / use case の純粋ロジックを最優先**でユニットテストする
2. port はモックまたはテストダブルで差し替え可能にする
3. Tauri コマンドの薄いラッパは、必要なら結合寄りで最小限
4. フレイキーな OS 依存テストは避け、抽象経由で検証する

## Commands

```sh
cd tauri-desktop/rust-native
cargo test
cargo test -- --nocapture
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

ルートから:

```sh
task rust:test
# または
cd tauri-desktop && cargo test
```

## Test File Organization

### モジュール隣接（推奨・デフォルト）

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rejects_empty() {
        assert!(GuestName::parse("").is_err());
        assert!(GuestName::parse("   ").is_err());
    }

    #[test]
    fn parse_trims_and_accepts() {
        let name = GuestName::parse("  Alice  ").expect("valid");
        assert_eq!(name.as_str(), "Alice");
    }
}
```

### 統合テスト

大きくなったら `tauri-desktop/rust-native/tests/` に配置を検討する。

## Arrange / Act / Assert

```rust
#[test]
fn greet_returns_message_for_valid_name() {
    // Arrange
    let input = "ReadyType";

    // Act
    let result = greet_use_case::execute(input);

    // Assert
    let message = result.expect("should succeed");
    assert!(message.contains("ReadyType"));
}
```

## Table-Driven Tests

```rust
#[test]
fn guest_name_parse_cases() {
    struct Case {
        input: &'static str,
        ok: bool,
        expected: Option<&'static str>,
    }

    let cases = [
        Case {
            input: "Alice",
            ok: true,
            expected: Some("Alice"),
        },
        Case {
            input: "  Bob  ",
            ok: true,
            expected: Some("Bob"),
        },
        Case {
            input: "",
            ok: false,
            expected: None,
        },
        Case {
            input: "   ",
            ok: false,
            expected: None,
        },
    ];

    for case in cases {
        let result = GuestName::parse(case.input);
        if case.ok {
            let name = result.unwrap_or_else(|_| panic!("expected Ok for {:?}", case.input));
            assert_eq!(name.as_str(), case.expected.expect("expected value"));
        } else {
            assert!(result.is_err(), "expected Err for {:?}", case.input);
        }
    }
}
```

名前付きケースが必要なときは `#[test]` をケースごとに分けるか、`rstest` 導入は承認必須。

## Port のテストダブル

```rust
struct FakePlatform {
    fail: bool,
}

impl PlatformGateway for FakePlatform {
    fn sync_window_chrome(&self) -> Result<(), PlatformError> {
        if self.fail {
            Err(PlatformError::Unavailable)
        } else {
            Ok(())
        }
    }
}

#[test]
fn use_case_propagates_platform_error() {
    let platform = FakePlatform { fail: true };
    let result = some_use_case::execute(&platform);
    assert!(result.is_err());
}
```

- application は trait にだけ依存する
- テストで具象 OS API を叩かない（可能な限り）

## Presentation（コマンド）

- コマンドは薄いので、原則 use case のテストで十分
- エラーマップ（`DomainError` → `String`）を変えたり契約を足したら、マップ用の小さなユニットテストを追加してよい
- フルの Tauri ランタイム結合は必須にしない

## What to Test / Not Test

### Test

- Value Object のパース・不変条件
- use case の成功・失敗パス
- エラー変換
- port 経由の分岐

### Avoid

- 生成コードやボイラープレートのみのファイル
- フレーム単位の UI（フロント Vitest）
- 実 OS ウィンドウ操作に依存する不安定テスト（抽象化してダブルを使う）

## Coverage Goals（目安）

- Domain: 高い（不変条件を網羅）
- Application: 高い（成功・代表的失敗）
- Infrastructure: 重要な変換・分岐のみ
- Presentation: 必要最小限

## Anti-Patterns

- `unwrap` だらけで失敗理由が分からないテスト
- テスト間で共有ミュータブル状態
- 本番コードにテスト専用の公開 API を増やす
- フロントと重複する IPC 契約テストを両側で過剰に持つ

## Checklist（新機能）

1. Domain / VO の異常系をテーブルまたは個別テストで追加
2. Use case の成功・失敗を追加
3. 新しい port なら Fake を用意
4. `cargo test` と `cargo clippy --all-targets -- -D warnings` を通す
