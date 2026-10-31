# 入力ソース自動切り替えの処理フロー

この文書は、現在のRust実装をもとに、macOSで入力欄にフォーカスしてから入力ソースを切り替えるまでの処理をまとめたものです。設定画面の操作方法ではなく、実装の責務と判断順を説明します。

## 全体像

```text
アプリの前面化
  ↓
Accessibilityでフォーカス変化を監視
  ↓
フォーカス要素と祖先要素の情報を取得
  ↓
重複・古いフォーカスイベントを整理
  ↓
有効状態・一時停止・ルールを確認
  ↓
優先順位順に最初に一致するルールを選択
  ↓
ルールの対象入力ソースを解決
  ↓
現在の入力ソースを確認し、TISで選択
  ↓
結果を検証・記録し、状態を通知
```

## 1. 前面アプリとフォーカスを監視する

macOSのフォーカス監視は、`NSWorkspaceDidActivateApplicationNotification`で前面アプリの切り替わりを受け取ります。アプリが前面化すると、そのプロセスに対してAccessibility Observerを登録し、`AXFocusedUIElementChanged`を監視します。

通知を受けると、その場で切り替え処理をせず、監視ワーカーのキューに評価要求を送ります。フォーカス通知に対応できないアプリでは50ms間隔のフォールバックポーリングを使い、通常の通知監視中も2秒ごとに状態を照合します。

Accessibility権限がなければ監視を停止します。権限状態は1秒ごとに確認し、許可されたら前面アプリの監視を再開します。

主な実装: [`macos/focus_monitor.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/macos/focus_monitor.rs)、[`focus_monitor/mod.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/focus_monitor/mod.rs)

## 2. フォーカス要素の情報を取得する

評価要求を処理すると、アクティブなプロセスのAccessibilityツリーからフォーカス要素とその祖先要素を取得し、`FocusSnapshot`を作ります。ルール設定に必要な属性を中心に取得します。通常の属性計画にはRole、Title、Description、Role Description、Placeholder、Identifier、DOM Identifier、DOM Class Listなどが含まれます。

取得結果が空の場合は通常のフォーカスイベントを発行しません。Chromium系アプリではAccessibilityツリーが非同期に構築されることがあるため、未初期化ならツリーを起こして再取得します。前面化時に必要な初回取得も行います。

主な実装: [`macos/focus_monitor.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/macos/focus_monitor.rs)、属性計画は[`switcher_controller/mod.rs`](../tauri-desktop/rust-native/src/application/switcher_controller/mod.rs)

## 3. 同じフォーカスや古いイベントを整理する

監視側は、フォーカス要素から最大6階層までのRole、DOM Identifier、DOM Class List、Identifierを組み合わせたfingerprintを作ります。前回と同じfingerprintなら新しいフォーカスイベントとして扱わず、generationを進めません。アプリが再び前面化した場合はfingerprintをリセットし、同じ入力欄でも再評価します。

実験用フラグ`READYTYPE_PROTO_DRAIN_FOCUS=1`を有効にした場合、ワーカーはキュー内の後続フォーカスイベントを最新のものにまとめます。このフラグは既定で無効です。switcher側では常にアクティブアプリ、PID、generationを確認し、別アプリのイベントや古いイベント、処理済みのgenerationを無視します。

主な実装: [`focus_monitor/mod.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/focus_monitor/mod.rs)、[`switcher_controller/mod.rs`](../tauri-desktop/rust-native/src/application/switcher_controller/mod.rs)、[`input_source_switcher/mod.rs`](../tauri-desktop/rust-native/src/application/input_source_switcher/mod.rs)

## 4. 自動切り替えの条件を確認する

`prepare_focus`は次の順に評価します。

1. イベントが現在のアプリとgenerationに対応し、まだ処理されていないこと
2. 手動変更を同じフォーカス上で尊重する状態に該当しないこと（該当時は切り替えを抑止）
3. 自動切り替えが有効であること
4. 一時停止期間中ではないこと
5. 登録ルールのいずれかがフォーカス情報に一致すること
6. 一致したルールの対象入力ソースが解決できること

ルールはpriority順で確認し、最初に一致した有効ルールを使います。ルールは対象プラットフォームとアプリIDが一致し、すべての条件が一致する必要があります。条件はFocused OnlyまたはFocused and Ancestorsの範囲で、属性値のEqualsまたはContainsにより照合されます。照合では属性値と設定値を小文字化して比較します。

対象が特定入力ソースならそのIDを使います。優先言語指定なら、設定済みの言語優先入力ソースへ解決します。ルールがない場合、切り替えは行いません。

主な実装: [`matcher.rs`](../tauri-desktop/rust-native/src/domain/input_source/matcher.rs)、[`input_source_switcher/mod.rs`](../tauri-desktop/rust-native/src/application/input_source_switcher/mod.rs)

## 5. 現在の入力ソースを確認して切り替える

ルール照合はswitcherの状態ロック内で行い、OS APIの呼び出しはロックを解放してから実行します。現在の入力ソースを読み、目標と異なる場合はmacOSのTISを使って目標を選択します。現在の入力ソースには有効なキャッシュを利用し、選択に失敗した場合はキャッシュを無効化します。

最初の選択が失敗した場合は、入力ソース一覧を再取得してからもう一度選択を試します。成功時には現在値を読み直し、目標と一致したかを記録します。現在値のキャッシュを使う経路は`READYTYPE_PROTO_CURRENT_CACHE=1`が必要で、既定では無効です。現在すでに目標と同じ場合も、通常設定では再選択します。これは速度より、フォーカス中フィールドの入力コンテキストも更新する安定性を狙った処理です。

主な実装: [`input_source_switcher/mod.rs`](../tauri-desktop/rust-native/src/application/input_source_switcher/mod.rs)、TIS実装は[`macos/tis.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/macos/tis.rs)

## 6. 選択後の遅延再選択

TISの選択が成功しても、メニューバーの表示が先に変わり、フォーカス中フィールドの入力コンテキストが後から更新される場合があります。このずれに備えて、TIS選択後に40ms、120ms、250ms後の再選択を予約します。

新しい選択が始まるとepochが更新され、古い選択に属する遅延コールバックは実行されません。各再選択ではTISの結果と現在の入力ソースを診断用に記録します。

主な実装: [`macos/tis.rs`](../tauri-desktop/rust-native/src/infrastructure/platform/macos/tis.rs)

## 7. 結果を状態に反映する

切り替え結果は`Switched`、`AlreadySelected`、`NoRule`、`Disabled`、`Snoozed`、`Ignored`、`Failed`などの判断として状態に反映されます。診断ログには対象アプリ、フォーカス情報、ルール、現在値と目標値、処理時間、失敗理由が記録されます。更新された状態はswitcher eventとしてフロントエンドへ通知されます。

## 補足

- 自動切り替えの入力欄識別はAccessibility属性の組み合わせに基づきます。すべてのアプリが同じ属性を公開するわけではありません。
- Accessibility権限がない場合、入力欄情報を取得できず自動切り替えは動作しません。
- Chromium/Electron系ではAccessibilityツリーの遅延構築やアプリごとの実装差を考慮した初期化・再取得を行います。
- この文書は実装の読み取り結果であり、将来の仕様を定めるものではありません。
