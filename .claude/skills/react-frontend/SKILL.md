---
name: react-frontend
description: >-
  ReadyType react-ui（features・TanStack Router・Jotai・Tauri invoke・Bulletproof）。
  UI 機能・レイアウト・共有コンポーネント実装時に使う。スタイル詳細は panda-css。
---

# React Frontend Skill

正本ルール: `.claude/rules/react-frontend.md`。スタイルは `panda-css` スキル / ルールも必須。

## スタック（変更は承認必須）

React 19 / Vite / TanStack Router / Panda CSS / Jotai / Vitest / Biome / Tauri `invoke`  
**未導入:** React Query / axios（追加は承認必須）

## ディレクトリ

```
src/features/<name>/   # components, hooks?, atoms?, types?, index.ts
src/components/<Name>/ # PascalCase
src/layouts/
src/routes/            # 薄い（スタイル禁止）
src/styles/            # Panda トークン
src/styled-system/     # 手編集禁止
```

## 実装チェックリスト

- [ ] ロジック・UI は `features/` または `components/`。`routes/` は組み立てのみ
- [ ] 外部 import は feature / component の `index.ts` 経由
- [ ] コンポーネント dir は PascalCase（`component-path-casing`）
- [ ] UI 状態は Jotai、永続/OS は `invoke`
- [ ] スタイルは隣接 `*.styles.ts`（`panda-css`）
- [ ] `any` / `unknown` なし
- [ ] UI/UX・依存追加は承認済みか確認

## invoke

```ts
import { invoke } from "@tauri-apps/api/core";
const message = await invoke<string>("greet", { name });
```

コマンド名は Rust と一致。秘密情報・重いロジックはフロントに置かない。

## 品質ゲート

```bash
cd tauri-desktop/react-ui
pnpm fix
pnpm typecheck
pnpm test
```

## 禁止

- `routes/` への `css` / ビジネスロジック
- feature 内部への横断 import
- SCSS / CSS Modules / vanilla-extract
- 承認なしの React Query / axios
