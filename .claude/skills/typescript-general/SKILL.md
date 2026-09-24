---
name: typescript-general
description: >-
  ReadyType TypeScript / Biome（型安全・命名・セキュリティ・ワークフロー）。
  react-ui の TS/TSX 実装・レビュー時に使う。
---

# TypeScript General Skill

正本ルール: `.claude/rules/typescript-general.md`。

## 型

- `any` 禁止。`unknown` は安易に増やさない
- 公開 API は型を明示
- 排他 props は `exclusive-union-props`
- 不合理な `@ts-expect-error` 禁止（使うなら理由コメント）

## Biome（実設定）

- indent 2 / lineWidth 100 / double quotes / semicolons always
- `noExplicitAny` / `useExhaustiveDependencies` / `noDeprecatedImports` = error
- 除外: `styled-system`、`routeTree.gen.ts`

```bash
cd tauri-desktop/react-ui
pnpm fix
pnpm typecheck
```

## 命名

- コンポーネント dir: PascalCase（`component-path-casing`）
- hooks: `useXxx`
- styles: `Name.styles.ts`
- alias: `@/`

## セキュリティ

- 秘密情報をハードコード・ログしない
- `dangerouslySetInnerHTML` に生ユーザー入力を渡さない
- 永続処理の本検証は Rust 側も前提

## 禁止

- 生成ファイルの手編集
- 承認なしの大規模依存追加
- 依頼外リファクタ（`agent-behavior`）
