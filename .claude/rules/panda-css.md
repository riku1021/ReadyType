---
description: Panda CSS（配置・css/cva・抽出・トークン・レイヤー）の実装規約
globs:
  - "tauri-desktop/react-ui/**/*.{ts,tsx}"
alwaysApply: false
---

# Panda CSS Rules

出典: [Writing Styles](https://panda-css.com/docs/concepts/writing-styles)、[Recipes](https://panda-css.com/docs/concepts/recipes)、[Cascade Layers](https://panda-css.com/docs/concepts/cascade-layers)、[Dynamic styling](https://panda-css.com/docs/guides/dynamic-styling)、[Tokens](https://panda-css.com/docs/theming/tokens)。

ビルド時に静的抽出して atomic CSS を生成する。実行時にスタイルオブジェクトを評価しない。

関連スキル: `panda-static-extraction` / `panda-style-layers`。

## 1. 配置

- 値（tokens / globalCss / conditions / keyframes）は `src/styles/`
- `panda.config.ts` はエンジン設定と配線のみ
- config から `@/styled-system` は import しない（codegen 循環）
- `styled-system/` は手編集しない。生成は `pnpm exec panda codegen` または `pnpm install`
- コンポーネントは `className={css(...)}`。`styled.div` や JSX style props での見た目指定は使わない（実行時 CSS 変数のみ例外）
- ページの見た目は `features/`（または `layouts/`）。`routes/` にスタイルを置かない

## 2. API

| import | 用途 |
|--------|------|
| `css` / `cva` / `cx` / `sva` | `@/styled-system/css` |
| `flex` / `stack` / `grid` など | `@/styled-system/patterns`（任意） |
| `token` / `token.var` | `@/styled-system/tokens`（`style` 属性・実行時 CSS 変数のみ） |

**config recipe**（`panda.config.ts` の `recipes`）は使わない。共通部品は `cva`。

## 3. css() / cva() の使い分け

| 使う | いつ |
|------|------|
| `css()` | 1 パターンの見た目。`_hover` / `_dark` / `mdDown` で足りる |
| `cva()` | `variant` / `size` など公開 API。実行時キーで見た目を切替 |
| `cx()` | **別プロパティ**のクラスを足す（`"group"`、開閉用） |
| `sva()` | 同じ variant を複数スロットで共有するときだけ |

### 置き場所

| 種類 | 置き場所 |
|------|----------|
| コンポーネント専用 | 隣接 `*.styles.ts`（例: `Button/Button.styles.ts`） |
| 横断（特定 UI の所有物でない） | `src/styles/objects/` |
| feature / layout ローカル | 隣接 `*.styles.ts` |

`routes/` には置かない。公開はコンポーネントの `index.ts` から re-export してよい。

### css() 例

```ts
import { css } from "@/styled-system/css";

export const header = css({
  display: "flex",
  height: "60px",
  backgroundColor: "background.sub",
  mdDown: { display: "none" },
});
```

子孫セレクタ（`& span`）は避け、対象要素に `css()` を付ける。

### cva() 例

```ts
import { cva } from "@/styled-system/css";

export const button = cva({
  base: { cursor: "pointer", fontWeight: "medium" },
  variants: {
    variant: {
      primary: { backgroundColor: "brand.primary.main", color: "white" },
      secondary: { backgroundColor: "background.sub", color: "brand.primary.main" },
    },
    size: { medium: { fontSize: "1rem" }, large: { fontSize: "1.125rem" } },
  },
  defaultVariants: { variant: "primary", size: "medium" },
});
```

- `cva` は **全 variant の CSS を常に生成**する。増やしすぎない
- 実行時マップは `css({ color: colorByType[type] })` にせず `cva` の `variants` にする

```ts
// ❌
css({ color: colorByType[type] });

// ✅
const tone = cva({
  variants: {
    type: {
      primary: { color: "brand.primary.main" },
      danger: { color: "status.error" },
    },
  },
});
tone({ type });
```

### 合成

同じプロパティを上書きするときは `cx(cva(), css())` ではなく `css()` でマージ（`cx` はクラス連結のみで勝敗は順序依存）。

```ts
css(button.raw({ variant: "secondary" }), { display: "inline-flex", gap: "xs" });

cx(styles.drawer, isOpen ? styles.drawerOpen : styles.drawerClosed);
```

## 4. 静的抽出

### 抽出できる

- 同一ファイルのリテラル・`as const` オブジェクト
- `css({ color: isOn ? "brand.primary.main" : "text.main" })`（両枝を生成）
- `css.raw({ ... })`
- `_hover` / `md` / `_dark` など

### 抽出できない（やらない）

```ts
css({ color });                          // state / props の値
css({ color: colorByType[type] });     // 実行時キー
css({ color: getColor() });            // 関数結果
css({ color: tokensFromOtherFile });   // 別ファイルの変数
css({ ...stylesFromOtherFile });       // 別ファイル object の spread → 欠落
```

**共有:** 定義側で `css.raw`（または `cva`）、利用側で `css(shared, { ...local })`。

```ts
// shared.ts
export const caret = css.raw({ appearance: "none", paddingRight: "1.75rem" });

// Feature.styles.ts
export const select = css({ display: "block", paddingInline: "sm" }, caret);
```

本当に実行時値が必要なら:

```ts
css({ color: "var(--accent)" });
// style={{ ["--accent"]: token.var(`colors.${key}`) }}
```

`staticCss` は最終手段。`['*']` や全 breakpoint 総当たりはしない。

## 5. トークンとテーマ

- トークンは必ず `{ value }`。生文字列を `theme.tokens` に直接置かない
- **tokens**: 変化しない基礎値（spacing、fonts、fontSizes、zIndex）
- **semanticTokens**: 意味とテーマで変わる値（colors）
- フォントは `fonts`（`fontFamilies` は無効）
- 単一ソースは `src/styles/tokens/`。`src/styles/panda.ts` 経由で config に載せる
- 汎用色は `styles/tokens/`、アプリ固有意味色は `styles/domains/<domain>/tokens/`

### 消費（推奨順）

```ts
css({ color: "brand.primary.main", padding: "sm" });
css({ boxShadow: "0 2px 4px {colors.shadow.primary}" });
```

1. `css` / `cva` のプロパティにトークン名
2. 複合値は `{path.to.token}` または文字列 `token(colors.x)`
3. JS の `token()` / `token.var()` は `style={{}}` や実行時 CSS 変数だけ

### テーマ

- アプリテーマは `data-theme="light" | "dark"`
- conditions で `_dark` / `_light` を上書き
- `_osDark` は OS 用。アプリテーマ切替には使わない
- レスポンシブは `md` / `mdDown`
- 論理プロパティ（`paddingInline` など）を物理より優先

### globalCss

- `src/styles/bases/global.ts` はプレーンオブジェクト
- リセットは `preflight: true`。`bases/reset.ts` は登録しない
- コンポーネント向けの見た目はここに書かない（html/body・変数・リセット相当のみ）

## 6. レイヤー

優先度: `reset < base < tokens < recipes < utilities`。`src/styles/index.css` の `@layer` はこの順。

- `globalCss` → `@layer base`（弱い）
- `css()` / `cva` / `_hover` など → `@layer utilities`（強い）
- **global から utilities を勝たせるために `!` / `!important` を使わない**
- **広すぎるセレクタ（`*` や `*:focus`）に色・枠を載せない**
- **レイヤー順の入れ替えや生成フックで base を utilities より強くしない**

詳細は `panda-style-layers` スキル。

## 7. 禁止

- SCSS / CSS Modules / vanilla-extract の新規追加
- `css({})` 内での JS `token()` 呼び出し（文字列 `token(colors.x)` や `{colors.x}` は可）
- トークンがある色・余白のハードコード
- 上書き目的の `!important`
- variant のない単一スタイルへの不必要な `cva`
- `css({ [runtimeKey]: ... })`
- config recipe の追加（承認なし）

## 8. コマンド

```bash
cd tauri-desktop/react-ui
pnpm exec panda codegen
pnpm fix
pnpm typecheck
```
