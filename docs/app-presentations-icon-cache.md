# アプリアイコン / 表示名の取得とキャッシュ

アプリ一覧・詳細・Quick Set などで使う OS アプリアイコンと Finder 風表示名の取得方針。

IPC: `get_app_presentations`
イベント: `app-presentations://icon-style-changed`

## 目標

| 目標 | 方針 |
|------|------|
| 画面遷移を速くする | FE セッションキャッシュ。未取得 id のみ IPC |
| 大きい画像を IPC に載せない | Native はキャッシュ PNG のパスを返し、FE は `convertFileSrc`（asset protocol） |
| OS の重い取得を繰り返さない | Native: memory → disk → `NSWorkspace.iconForFile` |
| アイコンスタイル変更に追随 | Native 監視 → イベント → FE キャッシュ破棄。再取得時は `?v=revision` で WebView 画像キャッシュも回避 |

## データフロー

```text
FE useAppPresentations(appIds)
  → sessionCache（AppPresentationView: iconSrc + displayName）
  → 不足分だけ get_app_presentations
       → AppIconResolver（CachedAppIconResolver）
       → AppDisplayNameResolver（macOS のみ Cached で包む）
  → resolveAppIconSrc(path, iconRevision) → asset URL?v=…
```

- IPC DTO（`AppPresentationDto`）: ファイルパス + `iconRevision` + `displayName`
- UI 型（`AppPresentationView`）: 変換済み `iconSrc` + `displayName`（混ぜない）

## Native キャッシュ（アイコン）

実装: `infrastructure/platform/app_icon_cache`

1. **Memory**（プロセス寿命）
   ヒットかつスタイル一致かつファイル存在 → パスを返す（`probe_mtime` しない）
2. **Disk**（`$APPCACHE/app-icons/`）
   `{bundleId}.png` + `{bundleId}.json`（`mtimeSecs`, `styleToken`）
3. **OS**
   ミス時のみ `NSWorkspace.iconForFile` → PNG 保存

`revision` は `{mtimeSecs}:{styleToken}`。Memory には `mtime` / `style` を保持し、返却時に生成する。

### スタイルトークン

macOS `AppleIconAppearanceTheme`（System Settings → Appearance → Icon & widget style）。
欠落時は空文字（Default 相当）。

## 表示名キャッシュ

macOS のみ `CachedAppDisplayNameResolver` でプロセス寿命の memory キャッシュ。
非 macOS は `Unsupported` をそのまま使う（包まない）。

## 無効化（アイコンスタイル変更）

```text
icon_style::start_monitor(callback)   // 1 秒ポーリング（infra、Tauri 非依存）
  → lib.rs で emit(ICON_STYLE_CHANGED_EVENT)
  → FE ensureAppPresentationStyleInvalidation
  → sessionCache clear → 購読中の useAppPresentations が再取得
```

ホットパス（画面遷移）はイベント駆動のまま IPC なし。
検知をポーリングにしている理由: UserDefaults KVO はコールバック内 prefs 再入で落ちやすく、クロスプロセスでは forced-sync タイマーが必要になりがち（Tinycast / macos-liquid-glass の知見）。1 秒 1 キーの読みはコストが小さく、実装を単純に保てる。

アプリ更新（bundle mtime 変化）は、FE に載っていない id の初回取得、またはスタイル無効化後の再取得、プロセス再起動時の disk 照合で拾う。起動中のアプリ更新だけを push する監視は未実装（稀）。

## Tauri 設定

- `tauri.conf.json`: `assetProtocol.enable` + scope `$APPCACHE/app-icons/**`
- `Cargo.toml`: `tauri` feature `protocol-asset`

CSP は現状 `null`（asset 利用時の追加制限は入れていない）。

## 主な参照実装

| 層 | パス |
|----|------|
| Port | `rust-native/src/application/ports/app_icon.rs`（`ResolvedAppIcon`） |
| Use case | `application/use_cases/app_presentations/` |
| Icon cache | `infrastructure/platform/app_icon_cache/` |
| Display name cache | `infrastructure/platform/app_display_name_cache/` |
| Style monitor | `infrastructure/platform/macos/icon_style.rs` |
| Emit 配線 | `lib.rs`（composition root） |
| Command / イベント名 | `presentation/commands/app_presentations.rs` |
| FE cache / convert | `react-ui/src/features/inputSourceApps/appPresentations.ts` |
| Hook | `…/components/appSettingsShared.tsx`（`useAppPresentations`） |

## 関連

- アプリ設定 UI・ルール編集: [input-source-rule-maintenance.md](./input-source-rule-maintenance.md)
- LanguageTag / 入力ソースモデル: [rules/ja-JP/input-source-model.md](./rules/ja-JP/input-source-model.md)
