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
| i18next / react-i18next | UI 文言（`ja` / `en` / `zh`、preference `system`） |
| Tauri API         | ネイティブ連携（`invoke` / プラグイン）          |
| Vitest            | ユニットテスト                                   |
| Biome             | lint / format                                    |
| Bulletproof React | features モジュール化の指針                      |

※ React Query / axios は現状未導入。非同期の共有読みが増えたら導入を検討する（**承認必須**）。
※ 国際化の詳細は本ファイル §14 および `docs/rules/ja-JP/react-frontend-i18n.md`。

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
├── locales/              # i18next（ja / en / zh、i18n.ts）
├── routes/               # TanStack Router（薄い）
├── styles/               # Panda トークン・globalCss
├── styled-system/        # Panda codegen（手編集禁止）
├── utils/
├── main.tsx
└── routeTree.gen.ts      # 生成物
```

`public/pngs/` など Vite public 配下も静的アセットとして利用可。ユーザー向け文言は `locales/` 経由のみ（ハードコード禁止）。

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
6. ユーザー向け文言は i18n（`t()`）。ハードコード禁止
7. 依頼範囲外のリファクタをしない（`agent-behavior`）

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

## 14. 国際化 (i18n)

ReadyType の UI 文言は **i18next + react-i18next** で管理する。テーマの `system` と同様に、表示言語も **preference（保存値）** と **resolved（実際に使う言語）** を分離する。

### 翻訳ファイル構成

翻訳は `tauri-desktop/react-ui/src/locales/{locale}/` 配下の JSON に置く:

```
src/locales/
├── i18n.ts                 # 初期化・preference / resolve・localStorage
├── registry.ts             # locales/registry.json の FE ラッパ
├── registry.json           # FE SoT: 対応パック・既定・nativeName
├── ja/
│   ├── index.ts            # 型付け（CustomTypeOptions）+ export
│   └── translation.json    # メイン翻訳
├── en/
│   ├── index.ts
│   └── translation.json
└── zh/
    ├── index.ts
    └── translation.json
```

トレイ文言（Rust 専用。対応コード一覧の SoT も兼ねる）:

```
tauri-desktop/rust-native/i18n/
└── tray.json
```

- FE 対応言語: `registry.json` の `locales[].code`
- Rust 対応言語: `tray.json` のキー（registry は読まない）
- preference: `system` | 各側の対応パック
- 永続キー（フロント）: `readytype_ui_locale`（`LANGUAGE_STORE_KEY`）
- ネイティブ永続: `SwitcherSettings.ui_locale`

言語追加は **FE registry.json → rust-native/i18n/tray.json → locales/{code}/** の順。

`ja/index.ts` / `en/index.ts` / `zh/index.ts` で `i18next` の `CustomTypeOptions` を拡張し、`t()` のキーを型安全にする。

### 翻訳 JSON 構造

トップレベルは機能ドメインでグループ化する（例: `nav`, `common`, `settings`, `home`, `dialog`）。

**例: `src/locales/ja/translation.json`（抜粋）**

```json
{
  "nav": {
    "home": "ホーム",
    "settings": "設定",
    "backToList": "一覧に戻る"
  },
  "common": {
    "saveFailed": "保存に失敗しました",
    "ruleCount": "{{count}}個"
  },
  "settings": {
    "displayLanguage": "表示言語",
    "languageSystem": "システム",
    "languageJa": "日本語",
    "languageEn": "English",
    "languageZh": "中文"
  }
}
```

**キー追加時の必須手順**

1. 各 `locales/{code}/translation.json` に同じキーを追加する（片方だけは禁止）
2. コンポーネントでは `t('...')` のみを使う
3. **新しい言語パック**を足す場合は `locales/registry.json` と `rust-native/i18n/tray.json` も更新する

### preference と resolved

| 概念 | 値 | 用途 |
|------|-----|------|
| preference | `system` \| FE registry / Rust tray の packs | 設定 UI・localStorage・Rust 設定ファイルに保存 |
| resolved | 各側の packs | `i18n.changeLanguage`・トレイ文言・`UiLocaleCache` |

- preference が対応パック → resolved はそのまま
- preference が `system` → OS 表示言語を解決（Rust: `sys-locale`、フロント起動直後のフォールバック: `navigator.languages`）
- 対応外の OS 言語 → 各側の defaultResolved（FE: registry、Rust: `DEFAULT_RESOLVED_UI_LOCALE` + tray 必須キー）
- **UiLocale ≠ 入力の LanguageTag**: 入力ソース preference（`languagePreferences`）は別モデル。詳細は `docs/rules/ja-JP/input-source-model.md`

テーマ（`light` / `dark` / `system`）と同じ分離なので、設定ラベル用ヘルパも同様に置く:

- テーマ: `features/settings/themeLabels.ts`
- 言語: `features/settings/languageLabels.ts`

### コンポーネントでの i18n の使用

**基本:**

```tsx
import { useTranslation } from "react-i18next";

export function MyComponent() {
  const { t } = useTranslation();

  return (
    <div>
      <h1>{t("nav.settings")}</h1>
      <button type="button">{t("common.requestPermission")}</button>
    </div>
  );
}
```

**補間:**

```tsx
// JSON: "ruleCount": "{{count}}個"
<p>{t("common.ruleCount", { count: rules.length })}</p>
```

**設定の表示言語切替:**

```tsx
import { getLanguageOptionLabel } from "./languageLabels";

// 一般設定の行は現在の表示名（system は翻訳、各言語は registry.nativeName）を出して
// /general/language へ遷移する。一覧の選択は preference（"system" | registry packs）を
// setUiLocale → invoke("set_ui_locale")。反映は settings.resolvedUiLocale で i18n.changeLanguage
```

### ネイティブ（トレイ）との同期

- トレイラベルは Rust 側で **resolved** 言語だけを使う（`infrastructure/desktop/tray.rs`）
- `set_ui_locale` 成功後、設定 DTO は `uiLocale`（preference）と `resolvedUiLocale` の両方を返す
- publish / トレイ更新は **switcher mutex を握ったまま行わない**（ロック外で `publish(state, resolved)`）
- ウィンドウフォーカス時、preference が `system` なら OS 言語を再解決してトレイを更新する

### IPC / ACL

新規または変更した `invoke` コマンドは次をセットで更新する:

1. Rust: `#[tauri::command]` + `generate_handler!`
2. フロント: `features/inputSource/api.ts` 等の `invoke` 名
3. **`rust-native/permissions/input-source-switcher.toml` の `commands.allow`**（欠けると実行時に失敗する）

言語関連の例: `set_ui_locale` は allow リスト必須。

### i18n ベストプラクティス

1. **常に翻訳キーを使う**: ユーザー向けテキストをハードコードしない。ラベル、メニュー、メッセージ、プレースホルダー、ツールチップ、aria-label、確認ダイアログはすべて `t()`。
2. **ja / en / zh を同時更新**: 片方の JSON だけ変更しない。
3. **名前空間の整理**: 関連キーをトップレベルでグループ化する（`nav`, `common`, `settings`, …）。
4. **一貫した命名**: ネストはドット記法（例: `settings.displayLanguage`）。
5. **動的ラベルも i18n**: セレクト選択肢・ステータス文言もキー化する（`languageLabels` / `themeLabels` パターン）。
6. **preference を保存する**: `system` 選択時に解決後の `ja`/`en`/`zh` だけを永続化しない（「システム」の意味が消える）。
7. **トレイは resolved**: preference 文字列をそのままトレイに渡さない。
8. **型**: `isUiLocalePreference` / `resolveUiLocale` / `CustomTypeOptions` を使い、生文字列比較を避ける。

### 重要: ユーザー向けテキストのハードコード禁止

コンポーネント内に日本語・英語の表示文言を直接書かない。必ず `translation.json` に定義して `t()` で参照する。

```tsx
// ❌ WRONG
<span>設定</span>
<button type="button">保存に失敗しました</button>
<button type="button" aria-label="一覧に戻る" />

// ✅ CORRECT
<span>{t("nav.settings")}</span>
<button type="button">{t("common.saveFailed")}</button>
<button type="button" aria-label={t("nav.backToList")} />
```

このルールが適用されるもの:

- メニュー・ナビゲーション
- ボタン・トグル・セクション見出し
- プレースホルダー・空状態
- エラー / 成功メッセージ
- ツールチップ
- ARIA ラベル（`aria-label={t('...')}`）
- 確認ダイアログ
- トレイに出す文言（Rust 側の locale 分岐。フロントの JSON と意味を揃える）

### 作業チェックリスト（UI 文言を触るとき）

- [ ] `ja/translation.json` と `en/translation.json` の両方にキーを追加・更新した
- [ ] コンポーネントは `useTranslation` + `t()` のみ（ハードコードなし）
- [ ] preference を変える場合は `set_ui_locale` と ACL を確認した
- [ ] トレイ文言が絡む場合は resolved 経路とロック外 publish を崩していない
- [ ] `pnpm typecheck`（キー型）と必要なら Vitest を通した

## 15. Anti-Patterns

- `routes/` にスタイルやビジネスロジック
- feature 内部への横断 import
- フロントでの秘密情報・重いドメインロジック
- Panda 以外の CSS スタック追加
- 承認なしの React Query / axios
- `any` / `unknown`
- コンポーネント dir の camelCase（PascalCase 必須）
- UI 文言のハードコード（日本語・英語を JSX に直書き）
- `ja` だけ / `en` だけの翻訳更新
- preference `system` を永続化せず resolved だけ保存する
- トレイ更新中に `controller.settings()` などで switcher mutex を再取得する
- 新規 IPC を `permissions/*.toml` に追加し忘れる