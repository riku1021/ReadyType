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
| ユニット配置 | **モジュール隣接** `foo/mod.rs` + `foo/tests.rs`（`#[cfg(test)] mod tests;`。`src` 内） |
| 統合配置 | クレート直下 `tests/*.rs`（**公開 API のみ**。別クレート扱い） |
| Mock | 手書きテストダブル、または必要なら mockall 等（追加は承認必須） |

フロントの Vitest（`mockIPC`）と責務を分ける。Rust は domain / use case / infrastructure を、IPC 契約の UI 側は Vitest を担当する。

**FE の `react-ui/tests/` ミラー配置を Rust にコピーしない。**  
Cargo の `tests/` は Jest/Vitest の `tests/` と意味が違う（統合・ブラックボックス専用）。ユニットをそこに移すと private 検証ができず、テストのためだけの `pub` が増えやすい。

## 方針

1. **domain / use case の純粋ロジックを最優先**でユニットテストする（隣接）
2. port はモックまたはテストダブルで差し替え可能にする
3. Tauri コマンドの薄いラッパは、必要なら結合寄りで最小限
4. フレイキーな OS 依存テストは避け、抽象経由で検証する
5. 公開 API を横断するシナリオが必要になったときだけクレート `tests/` を追加する

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

参照: [The Rust Book — Test Organization](https://doc.rust-lang.org/stable/book/ch11-03-test-organization.html)

### ユニット（推奨・デフォルト）

対象モジュールと**隣接する `tests.rs`** に置く。`use super::*` で private を検証できる。

```text
src/foo/
  mod.rs              # 本番
  tests.rs            # #[cfg(test)] mod tests; から読む（クレート内部）
```

本番側（`mod.rs`）:

```rust
#[cfg(test)]
mod tests;
```

`tests.rs`:

```rust
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
```

空の `rust-native/tests/` を先に作らない。

### 統合テスト（公開 API・複数モジュール連携）

`tauri-desktop/rust-native/tests/*.rs` は **外からクレートを使う**テスト専用。例: 設定の読み書き＋スイッチ状態など、公開境界をまたぐシナリオ。

- 各 `tests/*.rs` は別クレートとしてコンパイルされる
- private / `pub(crate)` には届かない（届くように本番 API を広げない）
- 共有ヘルパは `tests/common/mod.rs`（`tests/common.rs` にしない）

ユニットで足りるロジックを統合に重複させない。

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
- FE の `tests/` ミラーに倣ってユニットをクレート `tests/` に移す
- 中身のない `tests/` ディレクトリだけを先に置く

## Checklist（新機能）

1. Domain / VO の異常系をテーブルまたは個別テストで追加
2. Use case の成功・失敗を追加
3. 新しい port なら Fake を用意
4. `cargo test` と `cargo clippy --all-targets -- -D warnings` を通す
