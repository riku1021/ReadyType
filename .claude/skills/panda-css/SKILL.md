---
name: panda-css
description: >-
  ReadyType Panda CSS（css/cva・トークン・静的抽出・レイヤー）。
  *.styles.ts・tokens・globalCss・見た目の不具合調査時に使う。
---

# Panda CSS Skill

正本ルール: `.claude/rules/panda-css.md`。  
補助: `panda-static-extraction` / `panda-style-layers`。

## 原則

- ビルド時静的抽出。実行時スタイルオブジェクト評価に頼らない
- `className={css(...)}` / `cva`。`styled.div` 禁止
- `styled-system/` 手編集禁止 → `pnpm exec panda codegen`
- `routes/` にスタイルを置かない

## css vs cva

| API | 用途 |
|-----|------|
| `css()` | 単一パターン |
| `cva()` | variant / size など公開切替 |
| `cx()` | 別クラス連結（上書きマージには使わない） |
| `sva()` | 複数スロットで同一 variant |

```ts
css(button.raw({ variant: "secondary" }), { gap: "xs" });
```

## 置き場所

- コンポーネント専用: 隣接 `*.styles.ts`
- 横断: `src/styles/objects/`
- トークン: `src/styles/tokens/`（domains 配下も可）

## 抽出 Do / Don't

```ts
// ✅
css({ color: on ? "brand.primary.main" : "text.main" });
css(sharedRaw, { padding: "sm" });

// ❌
css({ color: colorByType[type] });
css({ ...objectFromOtherFile });
```

共有は `css.raw` + `css(a, b)` または `cva`。

## レイヤー

`globalCss` = base（弱い）。`css`/`cva` = utilities（強い）。  
global から utilities を `!important` で殴らない。詳細は `panda-style-layers`。

## テーマ

- `data-theme="light"|"dark"` + `_dark` / `_light`
- アプリ切替に `_osDark` を使わない

## Done

```bash
cd tauri-desktop/react-ui
pnpm exec panda codegen
pnpm typecheck
```
