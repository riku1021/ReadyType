---
description: React UI（features・Router・Jotai・Tauri invoke・フォーム・モジュール境界）の実装規約
globs:
  - "tauri-desktop/react-ui/**/*.{ts,tsx}"
alwaysApply: false
---

# React Frontend Rules

対象: `tauri-desktop/react-ui/`。スタイリングの詳細は `panda-css` ルールを併読する。

## 1. 技術スタック

| 技術              | 用途                                             |
| ----------------- | ------------------------------------------------ |
| React 19          | UI                                               |
| Vite              | バンドラ + 開発サーバー（Tauri 連携・port 1420） |
| pnpm              | パッケージマネージャ（mise 固定）                |
| TanStack Router   | ファイルベースルーティング                       |
| Panda CSS         | ビルド時生成の型安全スタイリング                 |
| Jotai             | クライアント UI 状態（テーマなど）               |
| Tauri API         | ネイティブ連携（`invoke` / プラグイン）          |
| Vitest            | ユニットテスト                                   |
| Biome             | lint / format                                    |
| Bulletproof React | features モジュール化の指針                      |

※ React Query / axios は現状未導入。非同期の共有読みが増えたら導入を検討する（**承認必須**）。

## 2. ディレクトリ構成と役割

```
src/
├── assets/               # Lottie など静的アセット
├── atoms/                # グローバル共有の Jotai アトム
├── animations/           # 共通 Lottie ラッパ
├── components/           # 再利用 UI（ディレクトリは PascalCase）
├── features/             # 機能単位（Bulletproof React）
│   ├── home/
│   ├── settings/
│   ├── navigation/
│   └── notFound/
├── hooks/                # 共有カスタムフック
├── layouts/              # RootLayout / Drawer / Header
├── routes/               # TanStack Router（薄い）
├── styles/               # Panda トークン・globalCss
├── styled-system/        # Panda codegen（手編集禁止）
├── utils/
├── main.tsx
└── routeTree.gen.ts      # 生成物
```

`public/pngs/` など Vite public 配下も静的アセットとして利用可。

## 3. 機能モジュール（Bulletproof React）

```
features/機能名/
├── components/   # ページ UI（隣接 *.styles.ts）
├── hooks/        # 必要な場合
├── atoms/        # 必要な場合
├── types.ts      # 必要な場合
└── index.ts      # 公開 API
```

### ルール

- `routes/` は URL とページ組み立てのみ。`css()` / `*.styles.ts` は置かない
- 外部からの import は必ず `index.ts` 経由
- クライアント UI 状態は Jotai。Rust とのやりとりは `@tauri-apps/api` の `invoke`
- コンポーネント dir は PascalCase。手順は `component-path-casing` スキル
- feature 間の深い import（`features/a/.../内部`）は禁止。公開 API 経由

## 4. コンポーネント設計

### 配置

| 種類       | 場所                             |
| ---------- | -------------------------------- |
| 再利用 UI  | `src/components/ComponentName/`  |
| 機能固有   | `features/<feature>/components/` |
| レイアウト | `src/layouts/`                   |

### 公開 API

```ts
// components/Button/index.ts
export { Button } from "./Button";
export type { ButtonProps } from "./Button";
export { button } from "./Button.styles";
```

### Props

- `any` / `unknown` 禁止
- 相互排他 props は `exclusive-union-props` スキルに従う
- デフォルト props は破壊的に変えない

### フォーム

- ネイティブ `<form>` / 制御 or 非制御を状況に応じて選択
- 送信前にクライアント側で必要な検証を行う
- 重いバリデーションや永続化は Rust `invoke` 側でも行う
- ライブラリ（React Hook Form 等）の新規追加は **承認必須**

## 5. ルーティング（TanStack Router）

- ファイルベース: `src/routes/`
- ルートファイルは薄い（レイアウト選択 + feature コンポーネントのマウント）
- `routeTree.gen.ts` は手編集しない
- デスクトップ向け: 深い外部 URL 共有は主目的ではない。不要な 404 ページの過剰設計をしない（既存 `notFound` は維持してよい）

```tsx
// routes/index.tsx — 薄い例
import { createFileRoute } from "@tanstack/react-router";
import { HomePage } from "@/features/home";

export const Route = createFileRoute("/")({
  component: HomePage,
});
```

## 6. 状態管理

| 状態の種類                     | 手段                                              |
| ------------------------------ | ------------------------------------------------- |
| UI（テーマ、ドロワー開閉など） | Jotai atoms                                       |
| サーバー／永続／OS             | Rust + `invoke`（将来 Query 検討時は承認）        |
| フォームの一時入力             | ローカル state またはフォームライブラリ（承認後） |

- グローバル atom は `src/atoms/` または feature 内 `atoms/`
- atom の過剰なグローバル化を避ける

## 7. ネイティブ連携（Tauri）

- 読み書き・OS 操作は Rust コマンド（`tauri-desktop/rust-native`）へ `invoke`
- フロントに秘密情報や重いロジックを置かない
- HTTP が必要な場合は原則 Rust（`reqwest`）または `@tauri-apps/plugin-http`
- **axios 新規追加は承認必須**
- コマンド名は Rust の `#[tauri::command]` と一致させる

```ts
import { invoke } from "@tauri-apps/api/core";

const message = await invoke<string>("greet", { name });
```

- 失敗は UI でユーザー向けに扱う（Dialog 等）。生の内部エラーを出しすぎない

## 8. スタイリング（概要）

詳細は **`panda-css` ルール**。

- `className={css(...)}` / `cva`。`styled.div` や CSS Modules は使わない
- ページ見た目は `features/` / `layouts/` の隣接 `*.styles.ts`
- トークンは `src/styles/`。`styled-system/` は手編集禁止
- SCSS / vanilla-extract 禁止

## 9. モジュール共有・インポート

- path alias: `@/` → `src/`
- `styled-system` は `@/styled-system/...`
- feature の公開は `index.ts` のみ
- 循環依存を作らない

## 10. アクセシビリティ

- インタラクティブ要素はキーボード操作可能に
- アイコンボタンに `aria-label`
- ダイアログはフォーカストラップと Esc を考慮（既存 Dialog パターンに合わせる）
- 色だけに頼らない状態表示

## 11. UI/UX・依存の変更

以下は **ユーザー承認必須**:

- レイアウト構造・情報設計の大きな変更
- コンポーネントライブラリや CSS アプローチの変更
- React Query / axios / フォームライブラリ等の新規依存
- メジャーバージョンアップ

## 12. 設計原則

1. 機能モジュール化（高凝集・低結合）
2. プレゼンテーションとロジックの分離
3. 型安全性（`any` / `unknown` は使わない）
4. UI 状態（Jotai）とネイティブ／永続データ（Rust）の責務分離
5. アクセシビリティ（WAI-ARIA）
6. 依頼範囲外のリファクタをしない（`agent-behavior`）

## 13. コマンド

```bash
cd tauri-desktop/react-ui
pnpm install
pnpm dev
pnpm build
pnpm fix          # Biome
pnpm typecheck
pnpm test
pnpm exec panda codegen
```

## 14. Anti-Patterns

- `routes/` にスタイルやビジネスロジック
- feature 内部への横断 import
- フロントでの秘密情報・重いドメインロジック
- Panda 以外の CSS スタック追加
- 承認なしの React Query / axios
- `any` / `unknown`
- コンポーネント dir の camelCase（PascalCase 必須）
