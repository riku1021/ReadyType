---
name: rust-backend
description: >-
  ReadyType Rust native Onion Architecture（domain / application / presentation /
  infrastructure）。Tauri コマンド・ユースケース・Entity・インフラ実装時に使う。
---

# Rust Backend Skill

正本ルール: `.claude/rules/rust-backend.md`（同内容を実装時に適用する）。

## Onion 依存方向

```
presentation / infrastructure  →  application  →  domain
```

- `domain` / `application` は `tauri`・OS API・具象 DB を import しない
- 入口は `#[tauri::command]`（薄い）。処理は use case へ委譲

## 層チェックリスト

### Domain

- [ ] Entity / VO / DomainError のみ（IPC DTO なし）
- [ ] 不変条件は `parse` / コンストラクタで守る
- [ ] 貧血モデルにしない

### Application

- [ ] use case + port trait
- [ ] presentation / infrastructure を import していない
- [ ] `AppHandle` / `Window` を持ち込んでいない

### Presentation

- [ ] コマンドは薄い
- [ ] `commands/mod.rs` + `lib.rs` の `generate_handler!` に登録
- [ ] フロント `invoke` 名と一致
- [ ] DomainError → IPC 向けエラーへマップ

### Infrastructure

- [ ] port の具象実装のみ
- [ ] ドメインルールを書いていない
- [ ] OS 分岐はここ（または platform 配下）

## 新機能の手順

1. Domain（必要なら）
2. Application use case + port
3. Infrastructure 実装
4. Presentation コマンド
5. `generate_handler!` 登録
6. フロント `invoke` 接続
7. 品質ゲート

## 品質ゲート

```bash
cd tauri-desktop/rust-native
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
```

## 参照実装

- `src/lib.rs`
- `src/presentation/commands/greet.rs`
- `src/application/use_cases/greet.rs`
- `src/domain/entities/guest_name.rs`

## 禁止

- コマンドへのビジネスロジック直書き
- 内側の層から外側への依存
- 秘密情報のハードコード
