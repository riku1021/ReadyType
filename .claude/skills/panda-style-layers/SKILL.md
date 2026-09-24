---
name: panda-style-layers
description: >-
  Panda の cascade layers（base vs utilities）。globalCss で utilities を殴らない、
  !important / ユニバーサル見た目指定を避ける。globalCss・レイヤー触るときに使う。
---

# Panda スタイル置き場とレイヤー

`globalCss`（`@layer base`）は `css()` / `cva`（`@layer utilities`）より弱い。  
レイヤーに逆らうと意図しない挙動が起きやすい。

正本: `.claude/rules/panda-css.md` の「レイヤー」節。

## 置き場

| 置き場 | レイヤー | 置くもの |
|--------|----------|----------|
| `bases/global.ts`（globalCss） | base | html/body、CSS 変数、リセット相当 |
| コンポーネントの `css` / `cva` | utilities | 色・枠・余白・`_hover` / `_focus` |

繰り返しは **`css.raw` + `css(a, b)`** または `cva`。  
別ファイル素 object の spread は抽出漏れ（`panda-static-extraction`）。

## Don't

- globalCss にコンポーネント向け見た目（色・枠・状態付き色）
- `*` / `*:focus` など広すぎるセレクタに見た目
- utilities を勝たせるための `!` / `!important`
- `@layer` 順入れ替えや base を強くするハック

## 参照

- [Cascade Layers](https://panda-css.com/docs/concepts/cascade-layers)
- [Global Styles](https://panda-css.com/docs/concepts/global-styles)
