# AGENTS.md

Codex 向けのプロジェクト指示。詳細なコーディング規約の正本は `.claude/rules/` と `.claude/skills/`（Claude / Cursor と共有）。日本語で回答する。

## 役割の分け方（重要）

| 置き場 | 用途 |
|--------|------|
| この `AGENTS.md` | Codex が最初に読む短い契約（コマンド・方針・ルーティング） |
| `.claude/rules/` | 詳細なコーディング規約（正本） |
| `.agents/skills/` | Codex が発見するスキル（`.claude/skills` へのリンク） |
| `.codex/config.toml` | プロジェクト設定（trusted 時のみ読込） |
| `.codex/rules/*.rules` | **コマンド承認**（Starlark）。アーキテクチャ規約は書かない |

## Project Overview

ReadyType はタイピング練習用の **Tauri 2 デスクトップアプリ**。

- Frontend: React 19, Vite 8, TypeScript, TanStack Router, Panda CSS, Jotai, **i18next**, Biome, Vitest — `tauri-desktop/react-ui/`
- Native: Rust (Tauri), **Onion**（`domain` → `application` → `presentation` / `infrastructure`）— `tauri-desktop/rust-native/`
- Tooling: mise, pnpm, lefthook

## Commands

```bash
mise install
mise run install          # react-ui の pnpm install
mise run dev              # Tauri + Vite
mise run check            # typecheck
mise run lint / mise run fix
mise run test             # Vitest
mise run fmt / mise run clippy
cd tauri-desktop/rust-native && cargo test
lefthook install          # 初回のみ
```

## Architecture（要約）

### Frontend（Bulletproof `features/`）

- `src/features/`・`src/components/`（PascalCase）・`src/layouts/`・薄い `src/routes/`
- UI 状態: Jotai / ネイティブ: `invoke`（React Query / axios 追加は承認必須）
- スタイル: Panda（`*.styles.ts`）。`styled-system/` 手編集禁止
- 文言: i18next（`src/locales/`）。`t()` 必須。詳細は `.claude/rules/react-frontend.md` §14 / `docs/rules/ja-JP/react-frontend-i18n.md`
- 入力ソースモデル（LanguageTag ≠ UiLocale）: `docs/rules/ja-JP/input-source-model.md`

### Native（Onion）

- 外側 → 内側のみ。`domain` / `application` に `tauri`・OS API を持ち込まない
- `#[tauri::command]` は薄く、処理は use case へ

## 作業時のルーティング

実装前に該当スキルを使う（`$skill` または暗黙選択）:

| 領域 | Skills | Rules（詳細） |
|------|--------|----------------|
| Rust | `rust-backend`, `rust-testing` | `.claude/rules/rust-*.md` |
| React / UI | `react-frontend`, `panda-css` | `.claude/rules/react-frontend.md`, `panda-css.md` |
| TS / テスト | `typescript-general`, `typescript-testing` | `.claude/rules/typescript-*.md` |
| 振る舞い | `agent-behavior` | `.claude/rules/agent-behavior.md` |
| Panda トラブル | `panda-static-extraction`, `panda-style-layers` | （同上 panda-css） |
| パス / props | `component-path-casing`, `exclusive-union-props` | — |

## 必須の制約

- `any` / `unknown` 禁止。TypeScript で `class` は必要最小限
- ユーザー向け文言は i18n（ハードコード禁止）。`ja`/`en` を同時更新。locale preference は `system`|`ja`|`en`
- UI/UX・依存追加（React Query / axios / 別 CSS スタック）は承認必須
- 依頼範囲外のリファクタをしない（`agent-behavior`）
- 秘密情報をハードコードしない
- 新規 Tauri コマンドは `permissions/*.toml` の allow にも追加

## 生成物（手編集禁止）

- `tauri-desktop/react-ui/src/styled-system/`
- `tauri-desktop/react-ui/src/routeTree.gen.ts`

## 品質ゲート（変更後）

Frontend:

```bash
cd tauri-desktop/react-ui && pnpm fix && pnpm typecheck && pnpm test
```

Rust:

```bash
cd tauri-desktop/rust-native && cargo fmt && cargo clippy --all-targets -- -D warnings && cargo test
```
