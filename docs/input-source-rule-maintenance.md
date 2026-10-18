# 入力ソースルールのメンテ手順

ReadyType のルールは、前面アプリの bundle ID と AX 属性の AND 条件で判定する。
UI 上の編集は「アプリごとの設定」（`/apps`）から行う。

新規インストール時のルール一覧は空。製品向けの既定プリセット（Terminal / Cursor / VS Code 等）は同梱しない。
既存ユーザーが保存済みのルールはそのまま維持される。

## 推奨フロー

1. Quick Set または「アプリごとの設定」から対象アプリのルールを追加する。
2. 値が安定している最小属性を選ぶ（優先: `domIdentifier` → `description` → `role`）。
3. 優先順位はアプリ詳細の並びを崩さない（例: terminal → chat → rename → editor）。
4. 取得した生の AX 文字列は永続化しない。

## 比較演算

- `equals`: 完全一致（大小文字無視）
- `contains`: 部分一致（大小文字無視）
- `focusedOnly`: フォーカス要素のみ
- `focusedAndAncestors`: フォーカス要素と親方向

## リセット

一般設定の「すべてのルールを削除」でルール一覧を空に戻せる。

## プライバシー

フォーカス検知に使う title / description などは切替判定用途のみ。外部送信はしない。

## 関連

- アプリアイコン / 表示名の取得とキャッシュ: [app-presentations-icon-cache.md](./app-presentations-icon-cache.md)
