# AGENTS.md — Frontend (tauri-desktop/react-ui)

このディレクトリでは React UI を扱う。詳細はリポジトリ直下の `AGENTS.md` と `.claude/rules/`。

## 優先スキル

- `react-frontend`
- `panda-css`（`*.styles.ts` / tokens / globalCss を触るとき）
- `typescript-general`
- `typescript-testing`（テスト追加時）
- 必要なら `panda-static-extraction` / `panda-style-layers` / `component-path-casing`

## ローカル制約

- `routes/` は薄い（スタイル・ビジネスロジック禁止）
- feature の公開は `index.ts` 経由
- `styled-system/` と `routeTree.gen.ts` は手編集禁止
- 完了前: `pnpm fix && pnpm typecheck && pnpm test`
