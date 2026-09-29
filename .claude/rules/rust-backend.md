---
description: Rust ネイティブ（オニオンアーキテクチャ）の全体方針・層構成・実装規約
globs:
  - "tauri-desktop/rust-native/**/*.rs"
alwaysApply: false
---

# Rust Backend Rules（Onion Architecture）

クレート: `tauri-desktop/rust-native/`。クリーンアーキテクチャの用語ではなく **オニオン** として層を守る。

## Architecture Overview

```
presentation / infrastructure  （外側）
        ↓ 依存してよい
    application
        ↓
      domain                   （内側・核心）
```

| 層 | 役割 | 配置 |
|----|------|------|
| `domain` | Entity / VO / DomainError / Repository trait（必要な場合） | `src/domain/` |
| `application` | Use case + outbound port（trait） | `src/application/` |
| `presentation` | Tauri コマンド・IPC DTO・エラーマップ | `src/presentation/` |
| `infrastructure` | OS / DB / 外部 API 実装（port の実装） | `src/infrastructure/` |
| composition | `lib.rs` / `main.rs` で配線 | クレートルート |

**依存ルール:** 外側 → 内側のみ。`domain` / `application` が `tauri`・OS API・具象 DB を import しない。

## Directory Structure

```
tauri-desktop/rust-native/src/
├── domain/
│   ├── entities/
│   ├── errors.rs
│   └── ...
├── application/
│   ├── ports/
│   └── use_cases/
├── presentation/
│   └── commands/
├── infrastructure/
│   └── platform/
├── lib.rs          # composition + invoke_handler
└── main.rs
```

## Domain Layer

### 置いてよいもの

- Entity / Value Object（例: `GuestName`）
- DomainError と変換
- Repository など **純粋な trait**（永続化の抽象。実装は infrastructure）

### 置いてはいけないもの

- `tauri` / `serde` のコマンド属性・IPC 専用 DTO
- OS API・ファイル・ネットワーク・DB ドライバ
- UI 文言の組み立て（必要なら application か presentation）

### ルール

- 不変条件はコンストラクタ / `parse` で守る（貧血症にしない）
- 外部クレート依存は最小（標準とドメインに必要なものだけ）
- `pub` は意図した API のみ
- ビジネスルールは Domain に集約する

### 例（Value Object）

```rust
// src/domain/entities/guest_name.rs
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuestName(String);

impl GuestName {
    pub fn parse(raw: &str) -> Result<Self, DomainError> {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return Err(DomainError::InvalidGuestName);
        }
        Ok(Self(trimmed.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
```

### 参照

- `src/domain/entities/guest_name.rs`
- `src/domain/errors.rs`

## Application Layer

### 構成

```
application/
├── ports/        # 外側へ向かう抽象（PlatformGateway など）
└── use_cases/    # ユースケース関数 / 構造体
```

### ルール

- domain のみに依存する（presentation / infrastructure を import しない）
- ユースケースは「やりたいこと」を表現し、OS 詳細は port 経由
- port は trait。実装は `infrastructure` に置く
- Tauri の `AppHandle` / `Window` を application に持ち込まない
- 1 ユースケース = 1 の明確な操作。肥大化したら分割する

### 例

```rust
// src/application/use_cases/greet.rs
use crate::domain::entities::guest_name::GuestName;
use crate::domain::errors::DomainError;

pub fn execute(name: &str) -> Result<String, DomainError> {
    let guest = GuestName::parse(name)?;
    Ok(format!("Hello, {}! You've been greeted from Rust!", guest.as_str()))
}
```

### Port 例

```rust
// src/application/ports/platform.rs
pub trait PlatformGateway {
    fn sync_window_chrome(&self) -> Result<(), PlatformError>;
}
```

### 参照

- `src/application/use_cases/greet.rs`
- `src/application/ports/platform.rs`

## Presentation Layer

### 役割

- `#[tauri::command]` の定義
- 引数の受け取りと戻り値のシリアライズしやすい形への変換
- DomainError → フロント向けエラー（現状は `String` など）のマップ

### ルール

- コマンド本体は薄く、`application::use_cases` を呼ぶ
- ビジネスルールをコマンドに書かない
- 新規コマンドは `commands/mod.rs` と `lib.rs` の `generate_handler!` に登録
- フロントの `invoke("name", …)` とコマンド名を一致させる
- IPC DTO が必要なら presentation に置き、domain 型を直接フロント契約にしない

### 例

```rust
#[tauri::command]
pub fn greet(name: String) -> Result<String, String> {
    greet_use_case::execute(&name).map_err(|error| error.to_string())
}
```

### 参照

- `src/presentation/commands/greet.rs`
- `src/presentation/commands/sync_window_chrome.rs`
- `src/lib.rs`

## Infrastructure Layer

### 役割

- `application::ports`（および domain の repository trait）の **具象実装**
- OS 差分（`platform/{macos,windows,linux}`）
- 将来の DB / ファイル / HTTP クライアント実装

### ルール

- ドメインルールをここに書かない（validation は domain / use case）
- cfg で OS 分岐してよいが、application から具象型を直接配らない
- Tauri Window 操作など UI シェル寄りの処理はここに閉じ、コマンドから呼ぶ
- 外部 I/O の失敗は適切なエラー型に変換して返す

### 参照

- `src/infrastructure/platform/`
- `src/application/ports/platform.rs`

## Composition Root

- `lib.rs` / `main.rs` で層を配線する
- `invoke_handler` / `generate_handler!` でコマンドを登録
- 依存の組み立てはここだけ。内側の層に DI フレームワークを持ち込まない

## Error Handling

- Domain: `DomainError`（ビジネス不変条件）
- Application: DomainError を伝播、または use case 固有エラー
- Presentation: フロント向けに `String` やシリアライズ可能な型へマップ
- Infrastructure: I/O エラーを port のエラー型へ変換

panic でビジネス失敗を表現しない。`Result` を使う。

## Code Quality

```sh
cd tauri-desktop/rust-native
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

- 不要な `unwrap` / `expect` を本番パスに置かない
- `clippy` 警告を無視しない（`-D warnings`）
- 公開 API は最小限

## Security

- 秘密情報をソースにハードコードしない
- フロントから渡る入力は domain / use case で検証する
- ファイル・ネットワークは必要最小権限で触る

## UI locale（トレイ / 設定）

表示言語 preference は `SwitcherSettings.ui_locale`（`system` | `ja` | `en`）。

- **resolved**（`ja` | `en`）は `SwitcherSettings::resolve_ui_locale`（`sys-locale`）で決める
- トレイ・`UiLocaleCache` は resolved のみ扱う
- state の publish / `tray::update` は **switcher mutex 解放後**に行う（`settings()` 再ロック禁止）
- 新規 `#[tauri::command]` は `permissions/input-source-switcher.toml`（または該当 toml）の `commands.allow` に必ず追加する

詳細（フロント含む）: `docs/rules/ja-JP/react-frontend-i18n.md`、`.claude/rules/react-frontend.md` §14

## Testing

詳細は `rust-testing` ルール / スキルを参照。

- domain / use case の純粋ロジックをユニットテストで優先
- port はモックまたはテストダブルで差し替え可能にする
- コマンドの薄いラッパは必要なら結合寄りで最小限

## Anti-Patterns（禁止）

- domain / application から `tauri` を import
- コマンドにビジネスロジックを直書き
- infrastructure の具象型を application が直接 new して配る
- UI 文言の組み立てを domain に置く
- 層をまたいだ循環依存
- トレイ更新経路で switcher mutex を再取得する
- IPC を ACL（permissions）に登録し忘れる

## Adding a New Feature（チェックリスト）

1. Domain: Entity / VO / Error が必要か決める
2. Application: use case + 必要なら port trait
3. Infrastructure: port 実装（OS / I/O）
4. Presentation: 薄い `#[tauri::command]`
5. `lib.rs` の `generate_handler!` に登録
6. **permissions `commands.allow` に追加**
7. フロントの `invoke` 名と一致させる
8. ユーザー向け文言があれば `ja`/`en` の翻訳キーを追加（フロント規約）
9. `cargo test` / `cargo clippy` を通す

## Reference Implementations

- `src/lib.rs` — 層の説明と composition
- `src/presentation/commands/greet.rs`
- `src/application/use_cases/greet.rs`
- `src/domain/entities/guest_name.rs`
- `src/application/settings.rs` — `normalize_ui_locale` / `resolve_ui_locale`
- `src/infrastructure/desktop/tray.rs` — resolved locale でメニュー構築
