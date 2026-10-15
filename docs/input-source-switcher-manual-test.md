# 入力ソース自動切替 — 手動 E2E マトリクス

前提: macOS 13 以降、`mise run dev` で ReadyType を起動し、アクセシビリティ権限を付与できること。

| # | 前提 | 操作 | 期待 | 実績 | 遅延 |
|---|---|---|---|---|---|
| 1 | 権限あり・自動切替 ON | Terminal を前面化 | preferred English に切替 | _未実施_ | |
| 2 | Cursor 起動済み | 統合ターミナルへフォーカス | Cursor Terminal ルール | _未実施_ | |
| 3 | Cursor 起動済み | AI chat / composer へフォーカス | Cursor Chat → preferred Japanese | _未実施_ | |
| 4 | Cursor 起動済み | リネーム欄へフォーカス | Cursor Rename → preferred English | _未実施_ | |
| 5 | Cursor 起動済み | エディタへフォーカス | Cursor Editor → preferred English | _未実施_ | |
| 6 | VS Code 起動済み | terminal / chat / rename / editor | 各 VS Code プリセット | _未実施_ | |
| 7 | 任意アプリ | カスタムルールを追加して前面化 | 一致して切替 | _未実施_ | |
| 8 | ルール一致後 | 同一フォーカス内で手動切替 | 次のフォーカスまで尊重 | _未実施_ | |
| 9 | 権限なし | アクセシビリティ画面 | 権限案内・監視なし | _未実施_ | |
| 10 | 権限拒否→許可 | システム設定で許可して戻る | 再起動なしで監視開始 | _未実施_ | |
| 11 | AX 通知非対応アプリ | 前面化 | fallback 50ms 監視 | _未実施_ | |
| 12 | 監視中 | ウィンドウを閉じる | トレイ常駐・監視継続 | _未実施_ | |
| 13 | スヌーズ 5 分 | 期限待ち | 自動復帰 | _未実施_ | |
| 14 | `--autostart` | ログイン起動相当 | ウィンドウ非表示で起動 | _未実施_ | |
| 15 | 入力ソース削除 | 設定画面 | 利用不可表示 | _未実施_ | |
| 16 | 診断画面 | コピー / 条件取り込み / 離脱 | キャプチャ停止・生テキスト非永続化 | _未実施_ | |
| L1 | 自動切替 ON・監視中・Lab ルール準備済 | 開発確認 → 「ネイティブ入力窓を開く」→ 欄 01–06 を順にフォーカス | 奇数欄 preferred ja / 偶数欄 preferred en（メニュー＋実入力） | _未実施_ | |
| L2 | L1 のあと同一欄フォーカス維持 | メニューバーで手動 IME 切替 → 自動 1 キー相当の入力 | 手動ソースが維持され入力できる（ルールへ即戻らない） | _未実施_ | |
| L3 | L1 と同前提 | `mise run rust-native:lab-smoke`（Lab 窓が前面に出る） | AC-2 / AC-3 相当がパス。失敗時は stderr / `tmp/input-source-logs/` | _未実施_ | |

正本・ablation: [input-source-switch-test-plan.md](./input-source-switch-test-plan.md)

## 性能（診断画面）

- p50:
- p95:
- 測定条件:

## メモ

実機未実施の行は `Not run — <理由>` とし、通過扱いにしない。
