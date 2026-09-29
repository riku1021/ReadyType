# 国際化 (i18n) — ReadyType

AI エージェントおよび開発者向けの i18n ガイド。**コーディング規約の正本は** `.claude/rules/react-frontend.md` の「国際化 (i18n)」節。

## 責務の分離

| 成果物 | 場所 | 誰が読むか |
|--------|------|------------|
| UI 言語レジストリ（セレクト・既定・nativeName） | `react-ui/src/locales/registry.json` | **FE のみ** |
| UI 文言 | `react-ui/src/locales/{code}/translation.json` | **FE のみ** |
| トレイ文言＋ネイティブ対応コード | `rust-native/i18n/tray.json` | **Rust のみ** |

React はトレイを直接更新しない。`set_ui_locale` の結果として Rust が `tray::update` する。  
Rust の「対応パック」は **`tray.json` のキー**が SoT（FE の registry は読まない）。

## 言語を追加する手順

1. `react-ui/src/locales/registry.json` に `{ "code", "nativeName" }` を追加
2. `rust-native/i18n/tray.json` に同じ `code` のトレイ文言を追加
3. `react-ui/src/locales/{code}/` に `translation.json` + `index.ts` を追加
4. ビルド / テスト

## ディレクトリ

```
tauri-desktop/react-ui/src/locales/
├── registry.json           # FE SoT: 対応パック・既定・nativeName
├── registry.ts
├── i18n.ts
└── ja|en|zh/
    ├── index.ts
    └── translation.json

tauri-desktop/rust-native/i18n/
└── tray.json               # Rust SoT: トレイ文言 = 対応コード一覧
```

## preference と resolved

| | preference | resolved |
|--|------------|----------|
| 値 | `system` \| 各側の対応パック | 各側の対応パック |
| 保存先 | localStorage + `SwitcherSettings.ui_locale` | 保存しない |
| FE | `registry.json` | `i18n.changeLanguage` |
| Rust | 同上 preference 文字列 | `tray.json` キーで検証・OS 解決・トレイ |

- `system` → OS（Rust: `sys-locale` / FE 起動時: `navigator.languages`）
- 非対応 → 各側の defaultResolved（現状どちらも `ja`。FE は registry、Rust は定数＋tray に必須）

設定セレクトの言語名は **nativeName**（FE registry）。`system` だけ `settings.languageSystem` を翻訳。

**UiLocale ≠ LanguageTag（入力）**: 詳細は [input-source-model.md](./input-source-model.md)。

## コンポーネント

```tsx
import { useTranslation } from "react-i18next";

const { t } = useTranslation();
return <h1>{t("nav.settings")}</h1>;
```

## ハードコード禁止

- ウィンドウ内: `t()` + `locales/{code}/translation.json`
- トレイ: `rust-native/i18n/tray.json`（React から触らない）

## キー追加手順

1. 各 `locales/{code}/translation.json` に同じキーを追加
2. UI で `t('...')` を使う
3. `pnpm typecheck`

## ネイティブ / トレイ

- トレイは resolved + `tray.json`（`application/tray_strings.rs`）
- publish → トレイ更新は switcher mutex の外（React 非関与）

## ベストプラクティス

1. FE 一覧は `registry.json`、Rust 一覧は `tray.json`（共有 JSON に無理にまとめない）
2. 言語追加時は両方に同じ `code` を足す
3. preference を永続化し、resolved だけを保存しない
4. トレイ更新は Rust のみ

## 参照実装

- `react-ui/src/locales/registry.json` / `registry.ts` / `i18n.ts`
- `react-ui/src/features/settings/languageLabels.ts`
- `rust-native/i18n/tray.json`
- `rust-native/src/application/tray_strings.rs`
- `rust-native/src/infrastructure/desktop/tray.rs`
