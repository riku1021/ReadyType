# Input Source モデル（LanguageTag と InputSourceId）

AI エージェント向けのドメインメモ。入力ソース切替と UI 表示言語は別関心事である。

## 責務の分離

| 概念 | 型 | 説明 |
|------|-----|------|
| UI 表示言語 | `UiLocalePreference` (`system` \| 同梱パック) | i18n / トレイ。有限の翻訳パック |
| 言語タグ | `LanguageTag`（開いた文字列、BCP-47 primary） | 入力ソース preference のキー。閉 Union にしない |
| 入力ソース | `InputSourceId`（OS 固有の opaque ID） | macOS TIS ID / 将来の Windows TSF など |

`UiLocale` ≠ `LanguageTag`（入力）。韓国語入力を足しても UI 翻訳パックを足す必要はない。

## Preference マップ

```text
languagePreferences: Map<LanguageTag, Option<InputSourceId>>
  None 値 = unset（「設定しない」。Language の一種ではない）
```

- ドメイン・resolve は言語リストに依存しない（許可ホワイトリストなし）
- 空のとき起動時に OS からシード（現在の入力ソース言語。無ければ先頭言語グループ 1 つ）
- **設定画面では入力言語スロットを編集しない**（Preferences はルール解決用の内部データ。更新 IPC は持たない）

## RuleTarget

```text
PreferredLanguage(LanguageTag)  → preferences[tag] が fixed ならその ID
Exact(InputSourceId)            → その ID を直接使用
```

旧ワイヤ形式 `preferredEnglish` / `preferredJapanese` は読取時に `preferredLanguage` + `en`/`ja` へ正規化し、保存時は新形式のみ書く。

## 参照実装

- Rust: `domain/input_source/model.rs`（`LanguageTag`, `RuleTarget`）
- Rust: `application/settings.rs`（`language_preferences` / `seed_language_preferences_from_sources`）
- Rust: `application/input_source_switcher.rs`（`resolve_target`）
- TS: `features/inputSource/types.ts`
- TS: `features/inputSourceApps/appRulesViewModel.ts`

アプリ一覧のアイコン / 表示名キャッシュは [app-presentations-icon-cache.md](../../app-presentations-icon-cache.md)。
