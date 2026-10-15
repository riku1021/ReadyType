# 入力ソースルールのメンテ手順

ReadyType のルールは、前面アプリの bundle ID と AX 属性の AND 条件で判定する。
UI 上の編集は「アプリごとの設定」（`/apps`）から行う。

## 推奨フロー

1. ナビの「診断」を開き、デバッグキャプチャを開始する（ページ表示で自動開始）。
2. 対象アプリへフォーカスを移し、診断の AX 階層を確認する。
3. 値が安定している最小属性を選ぶ（優先: `domIdentifier` → `description` → `role`）。
4. 「アプリごとの設定」で対象アプリを開き、個別ルールとして条件と言語を保存する。
5. 優先順位はアプリ詳細の並びを崩さない（terminal → chat → rename → editor）。
6. 診断ページを離れるとキャプチャは停止する。取得した生の AX 文字列は永続化しない。

## 比較演算

- `equals`: 完全一致（大小文字無視）
- `contains`: 部分一致（大小文字無視）
- `focusedOnly`: フォーカス要素のみ
- `focusedAndAncestors`: フォーカス要素と親方向

## 既定プリセット

| アプリ | bundle ID |
|---|---|
| Terminal | `com.apple.Terminal` |
| Cursor | `com.todesktop.230313mzl4w4u92` |
| VS Code | `com.microsoft.VSCode` |

設定画面またはアプリごとの設定の「既定へ戻す」でプリセットにリセットできる。カスタムルールは消える。

## プライバシー

診断で見える title / description などはデバッグ用途のみ。外部送信やファイル保存はしない。

## 関連

- アプリアイコン / 表示名の取得とキャッシュ: [app-presentations-icon-cache.md](./app-presentations-icon-cache.md)
