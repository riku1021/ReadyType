# 入力ソース自動切替機能 設計書

作成日: 2026-09-24  
対象: ReadyType（Tauri 2 / React / Rust）  
初期対応OS: macOS 13以降

## 1. 目的

ReadyTypeを常駐アプリとして動作させ、前面アプリとフォーカス中のUI要素に応じて
入力ソースを自動切替する。既存PoCの全機能を移植しつつ、PoCの250msポーリングを
イベント駆動へ置き換え、フォーカス変化から入力ソース切替までの遅延を最小化する。

初期版ではmacOSを実装する。ドメインとユースケースをOS APIから分離し、将来の
Windows/Linuxアダプター追加を前提とする。

## 2. 成功条件

- Terminal、Cursor、VS Code向けの既定ルールが初回起動時に利用できる。
- 任意アプリ向けのルールを追加、編集、複製、並べ替え、無効化、削除できる。
- アプリ切替とアプリ内フォーカス移動をイベント駆動で検知する。
- 同じアプリ・同じ一致ルール内での手動入力ソース変更を、次のフォーカス変化まで尊重する。
- 自動切替のON/OFF、スヌーズ、入力ソース設定、権限案内、ログ、AXデバッグ、
  メニューバー常駐、ログイン起動を提供する。
- 通知受信からTIS切替要求完了まで、開発機でp95 50ms以内を目標とする。
- macOS以外でもビルド可能で、未対応状態をUIへ返す。

## 3. 非対象

- 初期版でのWindows/Linux固有監視・入力ソース切替実装
- App Store配布の保証
- IME辞書や変換候補の制御
- クラウド同期、テレメトリー、ログの外部送信
- OR条件、正規表現、条件グループなどの高度なルール式

## 4. アーキテクチャ

Rust側を機能の正本とし、Reactは設定と状態表示に限定する。フォーカスイベントごとに
WebView IPCを往復させない。

```text
NSWorkspace / AXObserver
          |
          v
macOS FocusMonitor adapter
          |
          v
Application switching service
          |
          +--> Domain rule engine
          |
          +--> InputSourceGateway --> macOS TIS
          |
          +--> SettingsRepository --> versioned JSON
          |
          +--> state/log event --> Tauri event --> React UI
```

### 4.1 Domain

OS、Tauri、ファイルシステムへ依存しない。

- `PlatformAppId`: OS種別とプラットフォーム固有アプリID
- `InputSourceId`: 空文字を拒否する入力ソース識別子
- `AxElementSnapshot`: フォーカス要素から親方向へ並べた属性スナップショット
- `RuleCondition`: 属性、比較演算、探索範囲、比較値
- `FocusRule`: ID、名前、有効状態、アプリID、優先順位、条件、切替先
- `RuleSet`: 優先順位順の最初の一致を返す
- `SwitchDecision`: 切替、既に一致、対象外、無効、一時停止などの判断結果

### 4.2 Application

以下のportだけに依存する。

- `FocusMonitor`: 前面アプリ・フォーカス変化の開始と停止
- `InputSourceGateway`: 一覧取得、現在値取得、切替
- `SettingsRepository`: 設定のロードと原子的保存
- `Clock`: スヌーズ期限と計測時刻
- `EventPublisher`: UI向け状態・ログ通知

アプリケーションサービスはイベントごとに候補ルールを評価し、必要な場合のみ
`InputSourceGateway`を呼ぶ。最後に処理したアプリ、フォーカス世代、一致ルールを保持し、
同一フォーカスの重複通知を抑止する。ユーザーが同じフォーカス内で入力ソースを手動変更しても
再切替しない。新しいフォーカスイベントまたは前面アプリ変更で再評価する。

### 4.3 Infrastructure

- macOS: NSWorkspace、AXObserver、TIS、アクセシビリティ権限
- JSON設定リポジトリ
- Tauriイベント発行
- Windows/Linux: 初期版では明示的な`Unsupported`を返すno-op実装

Apple APIのunsafe境界はmacOS infrastructure内の小さなモジュールへ閉じ込める。
AXには`objc2-application-services`の生成済みbindingを優先し、TISでbindingが不足する部分だけを
狭いFFIラッパーにする。

### 4.4 Presentation

Tauri commandは入力DTOの検証、use case呼び出し、IPCエラーへの変換だけを行う。
監視状態はTauri managed stateとして保持し、Reactへは状態変化とログ追加をeventで配信する。

## 5. macOS監視方式

### 5.1 前面アプリ変更

`NSWorkspace.shared.notificationCenter`の
`didActivateApplicationNotification`を購読し、bundle ID、表示名、PIDを取得する。
前面アプリが変わるたびに以前のAX observerを解除し、新しいPIDへobserverを登録する。

### 5.2 フォーカス変更

前面アプリのapplication AX elementへ`kAXFocusedUIElementChangedNotification`を登録する。
observerのrun loop sourceは専用の監視run loopで処理し、Tauri UIスレッドをAX問い合わせで
塞がない。通知callbackは重い処理を直接行わず、最新世代番号付きの評価要求を処理キューへ渡す。

キューは同一世代の重複だけをまとめ、時間ベースのデバウンスは行わない。古いPIDや世代の
要求は破棄し、アプリ切替直後の遅延イベントが誤切替を起こさないようにする。

### 5.3 フォールバック

AX通知登録が`notificationUnsupported`等で失敗した対象アプリだけ、前面にいる間は50ms周期で
フォーカスを確認する。正常に通知登録できるアプリでは常時高速ポーリングを行わない。
取りこぼし検出のため、通知方式でも低頻度の整合性確認を行う。

### 5.4 低遅延化

- bundle IDで候補ルールを先に絞る。
- 候補ルールが参照するAX属性だけを取得する。
- 優先順位の高いルールが確定した時点で親探索を終了する。
- 比較用文字列とルール索引を設定変更時に構築する。
- 利用可能なTIS入力ソースと切替対象参照をキャッシュする。
- 切替前に現在の入力ソースを比較し、同一ならTISを呼ばない。
- TIS選択失敗時だけ入力ソースキャッシュを更新して1回再試行する。
- デバッグ画面表示中だけ全対象AX属性を収集する。

## 6. ルールモデル

### 6.1 Rule

各ルールは以下を持つ。

- `id`: UI側で生成する重複しない文字列ID。Rust側で空文字と重複を拒否する。
- `name`: ユーザー向け表示名
- `enabled`: 有効／無効
- `platform`: `macos`、将来の`windows`、`linux`
- `appId`: macOSではbundle ID
- `priority`: 小さい値を優先。保存時に連番へ正規化する。
- `conditions`: 0個以上のAND条件
- `target`: `preferredEnglish`、`preferredJapanese`、または具体的な入力ソースID

条件が0件のルールは、アプリID一致だけで成立する。Terminalの既定ルールに使用する。

### 6.2 Condition

- 属性: role、subrole、role description、title、description、identifier、DOM identifier、help
- 演算子: equals、contains
- 範囲: focused element only、focused element and ancestors
- 比較値: 空文字不可
- 大文字小文字: 初期版では常に無視

各条件は指定範囲内のいずれかの要素に一致すれば成立する。全条件が成立したルールだけを
一致とする。複数ルールが一致した場合は、優先順位が最も高い有効ルールを採用する。

### 6.3 既定プリセット

- Terminal (`com.apple.Terminal`): 条件なし → preferred English
- Cursor (`com.todesktop.230313mzl4w4u92`)
  - integrated terminal → preferred English
  - AI chat/composer → preferred Japanese
  - rename field → preferred English
  - editor → preferred English
- VS Code (`com.microsoft.VSCode`)
  - integrated terminal → preferred English
  - chat/composer → preferred Japanese
  - rename field → preferred English
  - editor → preferred English

優先順位はterminal、chat、rename、editorの順とする。具体的な属性値はPoCの値を初期値とし、
すべてGUIから変更できる。

## 7. 設定と永続化

Tauriのアプリ設定ディレクトリに`settings.json`を保存する。

保存内容:

- `schemaVersion`
- グローバル有効状態
- preferred English / Japaneseの入力ソースID
- ルール一覧と順序
- デバッグログ有効状態
- スヌーズ期限（期限切れならロード時に破棄）

保存は同一ディレクトリの一時ファイルへ書き込み、flush後に置換する。保存成功後にだけ
メモリ上の確定状態を更新する。破損JSONは日時付きファイルへ退避し、既定設定で起動して
ユーザーへ警告する。`schemaVersion`ごとの明示的な移行関数を置く。

ログイン起動状態はTauri autostart pluginを正本とし、JSONへ二重保存しない。

## 8. 状態とログ

UIへ公開する状態:

- 自動切替の有効状態とスヌーズ期限
- OSサポート状態
- アクセシビリティ権限状態
- 現在のアプリ、フォーカス世代、一致ルール
- 現在と切替先の入力ソース
- 監視方式（AX notification / fallback polling / stopped）
- 直近の検知、判定、切替所要時間
- 直近のエラー

切替ログはメモリ上の最大200件のリングバッファとし、時刻、アプリ、ルール、切替元・先、
結果、理由、各処理時間を保持する。AXのtitleやdescription等の詳細値はデバッグ画面表示中または
デバッグログ有効時だけ保持し、永続化も外部送信もしない。

## 9. UI

### 9.1 ホーム

- 自動切替ON/OFF
- 権限不足時の案内と「権限を要求」「システム設定を開く」
- 現在のアプリ、一致ルール、現在・切替先入力ソース
- 監視方式と直近の切替結果

### 9.2 ルール

- 一覧、追加、編集、複製、削除、有効化
- 上下ボタンによる優先順位変更（ドラッグ用依存は追加しない）
- アプリID、条件、切替先のフォーム
- 現在のAXデバッグ属性から条件を取り込む導線
- 既定プリセットへ戻す操作は確認ダイアログを挟む

### 9.3 診断

- 切替ログ一覧とコピー、クリア
- フォーカス要素から親方向へのAX属性一覧
- 個別属性のコピーとルール条件への取り込み
- 検知、判定、切替の時間表示
- デバッグ取得の開始／停止

### 9.4 設定

- インストール済み入力ソースからpreferred English / Japaneseを選択
- 表示名と入力ソースIDの併記
- 利用不能になった保存済みIDの警告表示
- ログイン時に起動
- デバッグログ
- 既定設定へのリセット

既存の仮ホームと仮設定項目を上記の実機能へ置き換える。AboutとUpdate等の独立画面は維持する。
UIライブラリを追加せず、既存のReact、Jotai、Panda CSS、Tauri APIを使用する。

## 10. メニューバーとライフサイクル

- ウィンドウを閉じたときは非表示にし、監視を継続する。
- 明示的な「終了」でだけプロセスを終了する。
- 通常起動ではメインウィンドウを表示する。
- ログイン起動ではメインウィンドウを表示しない。
- メニューにON/OFF、5分・15分・1時間スヌーズ、再開、ホーム表示、設定表示、終了を置く。
- 状態として現在の対象アプリと最後に一致したルールを表示する。
- 左クリックまたは既定アクションでメインウィンドウを表示する。

ログイン起動にはTauri公式autostart pluginを使用する。将来Windows/Linuxでも同じUI契約を
維持し、プラットフォーム固有の登録方式はpluginへ委譲する。

## 11. 権限

起動時はプロンプトを強制表示せず権限状態を確認する。ユーザーが「権限を要求」を選んだ場合だけ
`AXIsProcessTrustedWithOptions`でOSプロンプトを要求する。権限がなければ監視と切替を開始しない。

システム設定から戻った時、またはReadyTypeが再アクティブになった時に権限を再確認し、許可済みなら
再起動なしで監視を開始する。権限が実行中に失われた場合はobserverを停止し、状態とログへ反映する。

## 12. エラー処理

- AX observer登録失敗: 対象アプリだけfallback pollingへ移行
- AX属性取得失敗: 該当属性を不一致として扱い、診断理由を残す
- 対象プロセス終了: 古いobserverと保留イベントを破棄
- 入力ソース切替失敗: キャッシュ更新後に1回だけ再試行
- 入力ソース消失: 設定値を保持したまま利用不可としてUI表示
- 設定保存失敗: 変更を確定せずUIへエラーを返す
- 設定破損: 元ファイルを退避して既定値で起動
- イベント購読切断: 再接続時に完全な現在状態を再取得
- macOS以外: `UnsupportedPlatform`を返し、OS APIを呼ばない

## 13. IPC契約

コマンド名はRustとTypeScriptで一致させ、presentation層を薄く保つ。

- `get_input_source_switcher_state`
- `set_input_source_switcher_enabled`
- `set_snooze`
- `resume_input_source_switcher`
- `list_input_sources`
- `list_rules`
- `create_rule`
- `update_rule`
- `delete_rule`
- `reorder_rules`
- `reset_default_rules`
- `update_input_source_preferences`
- `get_accessibility_permission`
- `request_accessibility_permission`
- `open_accessibility_settings`
- `start_debug_capture`
- `stop_debug_capture`
- `get_focus_snapshot`
- `list_switch_logs`
- `clear_switch_logs`
- `get_autostart_enabled`
- `set_autostart_enabled`

状態更新は単一のversion付きTauri eventで配信する。Reactは起動・再接続時に
`get_input_source_switcher_state`で完全状態を取得し、その後のeventを適用する。

## 14. テスト戦略

### 14.1 Rust domain

- equals / containsと大小文字無視
- focused only / ancestor探索
- 複数条件AND
- 優先順位と無効ルール
- 条件なしアプリルール
- Terminal / Cursor / VS Codeプリセット
- 不正なID、空条件値、重複ID

### 14.2 Rust application

port fakeを使って以下を検証する。

- 一致時の切替と対象外時のno-op
- 現在値と切替先が同じ場合のno-op
- 同一フォーカスの重複抑止
- 手動切替の尊重
- ON/OFFとスヌーズ、自動復帰
- 古いPID・世代イベントの破棄
- 切替失敗時のキャッシュ更新と1回再試行
- 設定保存失敗時のロールバック

### 14.3 Infrastructure

- JSON round-trip、schema移行、破損時退避、原子的置換
- macOS API値からdomain型への変換
- TISキャッシュの更新条件
- OS APIそのものは不安定な自動テスト対象にせず、薄い境界の変換をテストする

### 14.4 React

- IPCラッパーのコマンド名・引数・エラー伝播
- ルールフォームとDTO変換
- 優先順位変更
- 入力ソースの利用不可表示
- 表示ロジックは純粋関数を優先し、巨大なsnapshotテストは作らない

### 14.5 手動E2E

- Terminal前面化でpreferred English
- Cursor / VS Codeのterminal、chat、rename、editor
- 任意アプリのカスタムルール
- 同一フォーカス内での手動切替尊重
- 権限拒否、許可、実行中の権限喪失
- 通知非対応時のfallback
- ウィンドウを閉じた後の常駐とメニューバー操作
- ログイン起動時の非表示起動
- 入力ソース削除時のエラー表示
- 実測p50 / p95の診断表示

## 15. 品質ゲート

Frontend:

```bash
cd tauri-desktop/react-ui
pnpm fix
pnpm typecheck
pnpm test
pnpm build
```

Rust:

```bash
cd tauri-desktop/rust-native
cargo fmt
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

最後にmacOS実機で手動E2Eを実施し、自動化できない項目と実測値を報告する。

## 16. 参考資料

- [Apple: NSWorkspace.didActivateApplicationNotification](https://developer.apple.com/documentation/appkit/nsworkspace/didactivateapplicationnotification)
- [Apple: AXObserverCreate](https://developer.apple.com/documentation/applicationservices/1460133-axobservercreate)
- [Apple: AX notification overview](https://developer.apple.com/documentation/applicationservices/axnotificationconstants_h)
- [Apple: kAXFocusedUIElementChangedNotification](https://developer.apple.com/documentation/applicationservices/kaxfocuseduielementchangednotification)
- [Apple: AXIsProcessTrustedWithOptions](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions)
- [Tauri 2: Autostart plugin](https://v2.tauri.app/plugin/autostart/)
- [objc2-application-services: AXObserver](https://docs.rs/objc2-application-services/latest/objc2_application_services/struct.AXObserver.html)

