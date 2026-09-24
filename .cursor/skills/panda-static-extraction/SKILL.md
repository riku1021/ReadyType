---
name: panda-static-extraction
description: >-
  Panda CSS の静的抽出。別ファイルのオブジェクトを css({ ...spread }) しない、
  共有は css.raw + css(a, b) か cva。スタイルが消える・効かない・抽出漏れのときに使う。
---

# Panda 静的抽出

Panda はビルド時にソースを読んで CSS を出す。**別ファイルの値を `css({ ... })` に渡すと抽出されず、見た目が消える**（例: 下矢印の `backgroundImage` が無い）。

## Do

- 同一ファイル内のリテラル / `as const`
- 共有スタイルは定義側で `css.raw({ ... })`（または `cva`）し、利用側は `css(shared, { ...local })`
- 後勝ちでマージしたいプロパティは **後ろの引数**に置く（例: `paddingRight` を共有 caret 側に残す）

```ts
// styles/objects/surface.ts や components/Button/Button.styles.ts など、定義側で css.raw / cva
export const shared = css.raw({ appearance: "none", paddingRight: "1.75rem" });

// Feature.styles.ts
export const select = css({ display: "block", paddingInline: "sm" }, shared);
```

## Don't

```ts
// 別ファイルのプレーンオブジェクトを spread / 参照 → 抽出しない
css({ ...stylesFromOtherFile });
css({ color: tokenFromOtherFile });
```

- 実行時の変数・関数結果・動的キーを `css()` に渡す
- 「共有のため」にだけ別ファイルの素の style object を作って spread する

## 参照

- ルール版（ts/tsx で自動適用）: `.cursor/rules/frontend/panda-extraction.mdc`
- 公式: [Dynamic styling](https://panda-css.com/docs/guides/dynamic-styling)、[Merging Styles](https://panda-css.com/docs/concepts/merging-styles)
