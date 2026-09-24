---
name: panda-style-layers
description: >-
  Panda のスタイル置き場と cascade layers（base vs utilities）。
  globalCss で utilities を殴らない、!important / ユニバーサル見た目指定を避ける。
  globalCss・bases/global.ts・css()/cva・レイヤー・!important を触るときに使う。
---

# Panda スタイル置き場とレイヤー

`globalCss`（`@layer base`）は `css()` / `cva` / `_hover` / `_focus` など（`@layer utilities`）より弱い。レイヤー設計に逆らったり、広すぎるセレクタで見た目を載せると、意図しない挙動が起きやすい。

## 置き場

| 置き場 | レイヤー | 置くもの |
|--------|----------|----------|
| `bases/global.ts`（globalCss） | base | html/body、CSS 変数、リセット相当（例: `outline: none`） |
| コンポーネントの `css` / `cva` | utilities | 色・枠・余白・状態（`_hover` / `_focus` など）の見た目 |

繰り返しは **`css.raw` + `css(a, b)`** または `cva`（utilities 側）。別ファイルの素オブジェクトを `css({ ...spread })` しない（静的抽出されずスタイルが消える → `panda-static-extraction`）。

## Don't

- globalCss でコンポーネント向けの見た目（`borderColor`・`backgroundColor`・状態付きの色など）を書く
- `*` や `*:focus` など広すぎるセレクタに見た目を載せる
- utilities を勝たせるための `!` / `!important`
- `@layer` 順の入れ替えや `cssgen:done` で base を utilities より強くするハック

## 参照

- ルール版（ts/tsx で自動適用）: `.cursor/rules/frontend/panda-css.mdc`
- 公式: [Cascade Layers](https://panda-css.com/docs/concepts/cascade-layers)、[Global Styles](https://panda-css.com/docs/concepts/global-styles)、[Writing Styles](https://panda-css.com/docs/concepts/writing-styles)
