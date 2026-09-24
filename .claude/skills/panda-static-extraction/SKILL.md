---
name: panda-static-extraction
description: >-
  Panda CSS の静的抽出。別ファイル object を css({ ...spread }) しない、
  共有は css.raw + css(a, b) か cva。スタイル欠落・抽出漏れ時に使う。
---

# Panda 静的抽出

Panda はビルド時にソースを読んで CSS を出す。  
**別ファイルの値を `css({ ... })` に渡すと抽出されず、見た目が消える。**

正本: `.claude/rules/panda-css.md` の「静的抽出」節。

## Do

- 同一ファイル内のリテラル / `as const`
- 共有は定義側 `css.raw` / `cva`、利用側 `css(shared, { ...local })`
- 後勝ちマージは後ろの引数

```ts
export const shared = css.raw({ appearance: "none", paddingRight: "1.75rem" });
export const select = css({ display: "block", paddingInline: "sm" }, shared);
```

## Don't

```ts
css({ ...stylesFromOtherFile });
css({ color: tokenFromOtherFile });
css({ color: colorByType[type] });
css({ color: getColor() });
```

## 実行時値が必要なとき

```ts
css({ color: "var(--accent)" });
// style={{ ["--accent"]: token.var(`colors.${key}`) }}
```

`staticCss` は最終手段。`['*']` は使わない。

## 参照

- [Dynamic styling](https://panda-css.com/docs/guides/dynamic-styling)
- [Merging Styles](https://panda-css.com/docs/concepts/merging-styles)
