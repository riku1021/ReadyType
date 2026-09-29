# 国際化 (i18n) — ReadyType

AI エージェントおよび開発者向けの i18n ガイド。**コーディング規約の正本は** `.claude/rules/react-frontend.md` の「国際化 (i18n)」節。本ドキュメントは同内容を独立して参照しやすくしたもので、ReadyType 固有（`system` preference・トレイ・ACL）を含む。

構成・禁止事項の詳細度は go-ts-template の React フロントエンド規約の i18n 章に揃えている。

## スタック

- **フロント**: i18next + react-i18next（`tauri-desktop/react-ui`）
- **ネイティブ**: トレイ文言は Rust で resolved 言語により分岐（`sys-locale` で OS 解決）

## ディレクトリ

```
tauri-desktop/react-ui/src/locales/
├── i18n.ts
├── ja/
│   ├── index.ts            # CustomTypeOptions
│   └── translation.json
└── en/
    ├── index.ts
    └── translation.json
```

## preference と resolved

テーマ（`light` / `dark` / `system`）と同じ分離:

| | preference | resolved |
|--|------------|----------|
| 値 | `system` \| `ja` \| `en` | `ja` \| `en` |
| 保存先 | localStorage + `SwitcherSettings.ui_locale` | 保存しない（都度解決） |
| 用途 | 設定 UI の選択 | `i18n.changeLanguage`・トレイ・`UiLocaleCache` |

- `system` → OS 表示言語（Rust: `sys-locale`）。フロント起動直後のみ `navigator.languages` をフォールバックに使う
- 非対応 OS 言語 → `ja`

**UiLocale ≠ LanguageTag（入力）**: UI 表示言語は同梱翻訳パックの集合。入力ソース切替の `LanguageTag` / `languagePreferences` とは別レイヤ（詳細は [input-source-model.md](./input-source-model.md)）。

ヘルパ:

- `features/settings/languageLabels.ts`（設定セレクト）
- `features/settings/themeLabels.ts`（テーマ。同パターン）

## コンポーネント

```tsx
import { useTranslation } from "react-i18next";

const { t } = useTranslation();
return <h1>{t("nav.settings")}</h1>;
```

補間:

```tsx
t("common.ruleCount", { count: 3 });
```

## ハードコード禁止

ユーザー向け文字列（ラベル、ボタン、メッセージ、プレースホルダー、ツールチップ、aria-label、ダイアログ）は **すべて `t()`**。JSX に日本語・英語を直書きしない。

```tsx
// ❌
<span>設定</span>

// ✅
<span>{t("nav.settings")}</span>
```

## キー追加手順

1. `ja/translation.json` に追加
2. **同じキー**を `en/translation.json` に追加
3. UI で `t('...')` を使う
4. `pnpm typecheck` で型を確認

## ネイティブ / トレイ

- トレイは **resolved** のみ使用（`tray.rs`）
- 設定 DTO: `uiLocale`（preference）+ `resolvedUiLocale`
- state publish → トレイ更新は **switcher mutex の外**で行う（再入デッドロック防止）
- フォーカス時、preference が `system` なら OS を再解決

## IPC / ACL

`set_ui_locale` など新規コマンドは:

1. `#[tauri::command]` + `generate_handler!`
2. フロント `invoke` 名一致
3. `rust-native/permissions/input-source-switcher.toml` の `commands.allow` に追加

## ベストプラクティス（要約）

1. 常に翻訳キー（ハードコード禁止）
2. ja / en 同時更新
3. ドット記法のネストキーでドメイン分割（`nav`, `common`, `settings`, …）
4. 動的選択肢も i18n（`languageLabels` 等）
5. preference を永続化し、resolved だけを保存しない
6. トレイ・キャッシュは resolved
7. ACL を忘れない

## 参照実装

- `src/locales/i18n.ts`
- `src/features/settings/SettingsPage.tsx`
- `src/features/settings/languageLabels.ts`
- `rust-native/src/application/settings.rs`（`normalize_ui_locale` / `resolve_ui_locale`）
- `rust-native/src/infrastructure/desktop/tray.rs`
- `rust-native/permissions/input-source-switcher.toml`
