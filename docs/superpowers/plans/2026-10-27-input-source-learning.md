# 入力ソース自動割り当て学習 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 利用者が有効化した後、欄ごとの入力開始時ソースを直近50回記録し、同一ソースが40回以上なら次回フォーカス時に承認候補を提示する。

**Architecture:** Rust domain/application 層で欄識別、50件リングバッファ、候補状態を管理し、設定ルールとは別の JSON ストアへ保存する。Native のアクセシビリティ変更通知を入力開始シグナルとしてアプリケーションへ渡し、既存 Quick Set 窓を候補承認に再利用する。React 設定画面は学習の有効化と履歴リセットを提供する。

**Tech Stack:** Rust / Tauri 2 / serde / React 19 / TypeScript / Jotai / TanStack Router / Panda CSS / i18next / Vitest / Cargo test

**Spec:** `docs/superpowers/specs/2026-10-27-input-source-learning-design.md`

## Global Constraints

- 学習機能は初期状態で無効とし、設定画面で有効化された後に記録を始める。
- 欄ごとに入力開始時のソース ID を時系列で最大50件保持するリングバッファとする。
- 直近50件のうち同じソースが40件以上になった場合、その欄の候補を適格とする。
- 同じ欄に既存の手動ルールがある場合は候補を表示しない。
- 見送られた候補は再提案しない。却下状態は欄ごとに学習データとともに保存し、学習履歴リセットで消去する。
- 入力内容および欄のラベルをログに保存しない。学習データは設定ルールと分け、端末内に保存する。
- `any` / `unknown` 禁止。UI 文言は i18next の全言語パックを同時更新する。
- 新規 Tauri コマンドは `permissions/*.toml` の allow にも追加する。
- 生成物 `src/styled-system/` と `src/routeTree.gen.ts` は手編集しない。
- 追加依存は導入しない。必要に見える場合は実装前に選択肢と理由を出して確認する。
- 自動切替のフォーカス処理を学習処理・永続化・候補 UI のために待たせない。学習無効時は入力開始監視も登録しない。
- 入力開始イベントは非ブロッキングで学習専用キューへ渡す。ソース ID はイベント発生時点の既存 current-source cache を `try_lock` で読み、cache miss / lock contention 時はサンプルを破棄する。学習側から共有 gateway mutex を取得して自動切替の `select` と競合させない。集計・ファイル I/O は自動切替 worker で行わない。
- 学習ログの保存はイベントごとに同期実行せず、学習 worker 上でまとめて行う。保存失敗・worker 停止は自動切替を失敗させない。
- 候補判定・Quick Set 表示は対象フォーカスの自動切替完了後に行う。処理待ちの間に対象フォーカスが変わった候補は表示しない。

## Review Focus

- フォーカスだけの移動・再通知を入力開始として誤計上しない — Task 3 でフォーカス変更のみ、同一入力開始イベント重複のテストを追加する。
- 入力値・欄ラベルを学習ファイルへ漏らさない — Task 2 で保存 JSON の内容テストを追加する。
- 同じ AX 識別値が別アプリにある場合に学習を混同しない — Task 1 でアプリ ID を含むフィンガープリントのテストを追加する。
- 手動ルールがある欄や識別不能な欄に候補を出さない — Task 4 で両ケースの候補抑止テストを追加する。
- 候補ソースが OS から削除済みの場合に、選べない候補を表示しない — Task 4 で unavailable source の抑止テストを追加する。
- 学習処理が切替を遅らせない — Task 4 で切替 worker と学習 worker の分離、非ブロッキング投入、保存遅延中も切替が完了するテストを追加する。
- 学習候補 UI が切替に割り込まない — Task 5 で切替完了後の提示と古いフォーカス候補の破棄を検証する。

---

### Task 1: 学習ドメインと欄別リングバッファ

**Files:**
- Create: `tauri-desktop/rust-native/src/domain/input_source/learning.rs`
- Modify: `tauri-desktop/rust-native/src/domain/input_source/mod.rs`
- Test: `tauri-desktop/rust-native/src/domain/input_source/learning/tests.rs`

**Interfaces:**
- Consumes: `InputSourceId`, `PlatformAppId`, `FocusSnapshot`, `AxAttribute`.
- Produces: `FieldFingerprint`, `FieldLearningRecord`, `LearningLog::record_start`, `LearningLog::suggestion`.
- `FieldLearningRecord` は最大50件の `InputSourceId` 履歴と `dismissed` 状態を持つ。AX 生文字列をフィールドとして保持しない。
- `FieldFingerprint::from_snapshot(&FocusSnapshot) -> Option<FieldFingerprint>` はアプリ ID と許可された安定属性から生成し、安定属性不足時は `None`。
- `LearningLog::record_start(&mut self, field: FieldFingerprint, source: InputSourceId)` は末尾へ追加し、51件目で最古を捨てる。
- `LearningLog::suggestion(&self, field: &FieldFingerprint) -> Option<&InputSourceId>` は50件あり、最多ソース件数が40以上で、未却下の場合のみ返す。

- [ ] **Step 1: 境界テストを書く** — 不足履歴、49件、50件中39件、50件中40件、51件目の eviction、属性不足、アプリ ID 違い、却下済みを網羅する。
- [ ] **Step 2: domain テストを実行して失敗を確認する** — `cd tauri-desktop/rust-native && cargo test domain::input_source::learning`
- [ ] **Step 3: 学習型と deterministic なフィンガープリント生成を実装する** — 生 AX 文字列を保存値に含めず、現在の domain モジュールの不変条件に沿わせる。
- [ ] **Step 4: domain テストを再実行して通過を確認する** — `cd tauri-desktop/rust-native && cargo test domain::input_source::learning`

### Task 2: 学習設定と分離 JSON 永続化

**Files:**
- Create: `tauri-desktop/rust-native/src/application/ports/input_source_learning_repository.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/persistence/json_input_source_learning_repository/mod.rs`
- Create: `tauri-desktop/rust-native/src/infrastructure/persistence/json_input_source_learning_repository/tests.rs`
- Modify: `tauri-desktop/rust-native/src/application/ports/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/persistence/mod.rs`
- Modify: `tauri-desktop/rust-native/src/application/settings/mod.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/input_source_dto/mod.rs`
- Modify: `tauri-desktop/rust-native/src/lib.rs`
- Test: `tauri-desktop/rust-native/src/application/settings/tests.rs`

**Interfaces:**
- Consumes: Task 1 の `LearningLog`。
- Produces: `InputSourceLearningRepository::{load, save, clear}` と永続設定 `input_source_learning_enabled: bool`（default `false`）。
- setup が `app_config_dir()/input-source-learning.json` を repository に渡す。enable preference は既存 `input-source-switcher.json` の settings DTO に含める。

- [ ] **Step 1: repository / settings テストを書く** — round-trip、missing/corrupt file の復旧、clear、default disabled、保存 JSON に AX 生値・ラベル・入力文字列が存在しないことを検証する。
- [ ] **Step 2: Rust テストを実行して失敗を確認する** — `cd tauri-desktop/rust-native && cargo test json_input_source_learning_repository`
- [ ] **Step 3: repository port / JSON adapter / opt-in 設定を実装する** — 既存 `JsonSettingsRepository` の atomic save / backup 方針に合わせる。
- [ ] **Step 4: 初期化で学習 repository を構成し controller に注入する** — ログ破損時に通常設定や入力切替を巻き込まない。
- [ ] **Step 5: 対象テストを再実行して通過を確認する** — `cd tauri-desktop/rust-native && cargo test json_input_source_learning_repository && cargo test application::settings`

### Task 3: OS の入力開始シグナルと重複抑止

**Files:**
- Create: `tauri-desktop/rust-native/src/application/ports/input_start_monitor.rs`
- Modify: `tauri-desktop/rust-native/src/application/ports/mod.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/macos/focus_monitor.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/windows/focus_monitor.rs`
- Modify: `tauri-desktop/rust-native/src/lib.rs`
- Modify: `tauri-desktop/rust-native/src/infrastructure/platform/focus_monitor/mod.rs`
- Test: `tauri-desktop/rust-native/src/infrastructure/platform/focus_monitor/tests.rs`

**Interfaces:**
- Consumes: `FocusSnapshot`, 既存 current-source cache、Task 1 の `FieldFingerprint`。
- Produces: `InputStartEvent { snapshot: FocusSnapshot }` と OS monitor callback。イベント側では入力値を含めない。
- 学習有効時だけ購読を有効化する。callback はイベント時点の source cache を `try_lock` で読み、snapshot / source を bounded learning queue に non-blocking で渡して戻る。cache miss / lock contention / queue full はサンプルを破棄する。gateway mutex による source 取得、集計、永続化を callback / 自動切替 worker / monitor run loop で行わない。
- macOS は `AXObserver` の focused element value-changed 通知を、Windows は UI Automation text/value changed 通知を調査・使用する。通知値は永続化せず、単なる focus 通知や unchanged value は開始扱いにしない。

**Platform references:** Apple [`AXObserverAddNotification`](https://developer.apple.com/documentation/applicationservices/1462089-axobserveraddnotification?changes=l_4), [`kAXValueChangedNotification`](https://developer.apple.com/documentation/applicationservices/kaxvaluechangednotification?changes=l___1_8_1_5&language=objc); Microsoft [Handling Text-related Events](https://learn.microsoft.com/windows/win32/winauto/uiauto-handlingtextrelatedevents), [UI Automation Events Overview](https://learn.microsoft.com/windows/win32/winauto/uiauto-eventsoverview). UIA text-change events can be programmatic, so implementation must compare transient before/after state and avoid persisting text.

- [ ] **Step 1: monitor のイベント判定テストを書く** — focus-only、変更なし、同一編集の重複通知、focus context 切替後の新規入力、通知非対応要素を検証する。
- [ ] **Step 2: monitor テストを実行して失敗を確認する** — `cd tauri-desktop/rust-native && cargo test focus_monitor`
- [ ] **Step 3: macOS / Windows の AX/UIA monitor から値を保存せず入力開始 callback を出す** — transient な前後比較に限り、ソース文字列や入力内容をログ・イベント payload に含めない。
- [ ] **Step 4: 入力開始 callback を composition root で controller に配線する** — unsupported platform / unsupported element は通知せず学習対象外とする。
- [ ] **Step 5: 対象 monitor テストを再実行して通過を確認する** — `cd tauri-desktop/rust-native && cargo test focus_monitor`

### Task 4: 学習ユースケースと候補ライフサイクル

**Files:**
- Create: `tauri-desktop/rust-native/src/application/input_source_learning/mod.rs`
- Create: `tauri-desktop/rust-native/src/application/input_source_learning/tests.rs`
- Modify: `tauri-desktop/rust-native/src/application/mod.rs`
- Modify: `tauri-desktop/rust-native/src/application/switcher_controller/mod.rs`
- Modify: `tauri-desktop/rust-native/src/application/input_source_switcher/mod.rs`
- Create: `tauri-desktop/rust-native/src/application/input_source_learning/worker.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/commands/input_source_switcher.rs`
- Test: `tauri-desktop/rust-native/src/application/switcher_controller/tests.rs`

**Interfaces:**
- Consumes: `InputStartEvent`, Task 1 `LearningLog`, Task 2 repository, existing `FocusSnapshot::matches` / `FocusRule::matches`。
- Produces: `InputSourceLearning::record_start`, `pending_suggestion(snapshot)`, `approve_suggestion`, `dismiss_suggestion`, `clear_history`。
- `record_start` は preference disabled 時に no-op。manual rule が同じ snapshot に match した場合や field/source が識別・解決不能な場合は候補にしない。
- 入力開始イベントは専用学習 worker が受け取り、集計・repository 保存を行う。自動切替 worker から学習 worker の完了を join / wait せず、repository を同期呼び出ししない。永続化は学習 worker 内で複数更新をまとめる。
- イベント時点の source ID は callback が既存 current-source cache を `try_lock` で読み取って event に含める。cache miss / lock contention はサンプルを破棄し、学習 worker は共有 gateway mutex を使わないため、自動切替の `select` とロック競合しない。
- 学習 worker の開始・停止・保存失敗は自動切替の開始・完了に影響させない。学習キューが飽和した場合はイベントを落として切替を優先する。
- focus event は既存の自動切替を先に処理し、切替完了後に学習候補を再評価・通知する。表示直前に app と generation が現在の focus と一致することを確認し、古い候補は捨てる。
- 新規 Tauri commands: `set_input_source_learning_enabled(enabled)`, `clear_input_source_learning_history()`, `approve_input_source_learning_suggestion()`, `dismiss_input_source_learning_suggestion()`。

- [ ] **Step 1: use case / worker テストを書く** — disabled、識別不能、既存ルール、閾値到達前/後、次回フォーカスだけで提示、unavailable source、承認後の通常ルール登録、却下後再提示なし、履歴 reset を検証する。保存を意図的に遅らせても自動切替 worker が待たず完了し、queue full / cache miss 時も callback が待たずサンプルを破棄することをテストする。
- [ ] **Step 2: application テストを実行して失敗を確認する** — `cd tauri-desktop/rust-native && cargo test application::input_source_learning`
- [ ] **Step 3: 独立した学習 worker に入力開始イベント処理を追加する** — callback はイベント時点の source cache を非ブロッキングで読み、snapshot / source を bounded queue に non-blocking 投入して戻る。cache miss / queue full は落とす。学習 worker は gateway mutex を使わず、重複排除後に1入力開始1記録とし、集計・まとめ保存を自動切替 worker の外で行う。
- [ ] **Step 4: focus 候補通知を自動切替完了後に処理する** — worker の処理順を明示し、候補表示前に現在 focus の app / generation を照合する。候補評価・UI 通知が切替処理を待たせないことをテストする。
- [ ] **Step 5: candidate action と enable/reset commands を追加し、permissions allow に登録する** — approve は既存 quick-set helper で通常 `FocusRule` を作成、dismiss は field record に永続化する。
- [ ] **Step 6: controller / application テストを再実行して通過を確認する** — `cd tauri-desktop/rust-native && cargo test application::input_source_learning && cargo test application::switcher_controller`

### Task 5: Quick Set 候補承認 UI

**Files:**
- Modify: `tauri-desktop/rust-native/src/application/quick_set/mod.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/input_source_dto/mod.rs`
- Modify: `tauri-desktop/rust-native/src/presentation/commands/quick_set.rs`
- Modify: `tauri-desktop/react-ui/src/features/quickSet/types.ts`
- Modify: `tauri-desktop/react-ui/src/features/quickSet/api.ts`
- Modify: `tauri-desktop/react-ui/src/features/quickSet/QuickSetPage.tsx`
- Modify: `tauri-desktop/react-ui/src/features/quickSet/QuickSetPage.styles.ts`
- Modify: `tauri-desktop/react-ui/src/locales/ja/translation.json`
- Modify: `tauri-desktop/react-ui/src/locales/en/translation.json`
- Modify: `tauri-desktop/react-ui/src/locales/zh/translation.json`
- Test: `tauri-desktop/rust-native/src/presentation/input_source_dto/tests.rs`
- Create: `tauri-desktop/react-ui/src/features/quickSet/quickSetViewModel.test.ts`

**Interfaces:**
- Consumes: Task 4 の候補イベント/session と既存 Quick Set source list。
- Produces: `QuickSetSessionDto` の suggestion mode と提案 source ID。通常ホットキー Quick Set の動作は維持する。
- candidate event は threshold 到達時に学習 worker で保存し、対象 field の次 focus で既存 `quick-set` 窓へ通知する。フォーカス時の自動切替処理完了後に通知し、入力開始後の割り込みはしない。学習 worker / repository の遅延を待ってから切替する構造にはしない。

- [ ] **Step 1: DTO / view model の分岐テストを書く** — suggestion session の推奨ソース表示、normal session の既存表示、available option がない提案の抑止を検証する。
- [ ] **Step 2: Rust DTO と FE view model テストを実行して失敗を確認する** — `cd tauri-desktop/rust-native && cargo test presentation::input_source_dto` と `cd tauri-desktop/react-ui && pnpm test -- quickSetViewModel`
- [ ] **Step 3: Quick Set に提案・承認・見送りの状態を追加する** — 承認時は通常ルール登録後に閉じ、見送り時は dismiss command の成功後に閉じる。
- [ ] **Step 4: 3 locale の提案文言と必要なスタイルを追加する** — 新しい hardcoded user-facing strings は置かない。
- [ ] **Step 5: DTO / view model テストを再実行して通過を確認する** — 上記 Step 2 と同じコマンド。

### Task 6: 設定 UI、有効化、履歴リセット

**Files:**
- Modify: `tauri-desktop/react-ui/src/features/inputSource/api.ts`
- Modify: `tauri-desktop/react-ui/src/features/inputSource/types.ts`
- Modify: `tauri-desktop/react-ui/src/features/inputSource/hooks/useInputSourceState.ts`
- Modify: `tauri-desktop/react-ui/src/features/settings/SettingsPage.tsx`
- Modify: `tauri-desktop/react-ui/src/features/settings/SettingsPage.styles.ts`
- Modify: `tauri-desktop/react-ui/src/locales/ja/translation.json`
- Modify: `tauri-desktop/react-ui/src/locales/en/translation.json`
- Modify: `tauri-desktop/react-ui/src/locales/zh/translation.json`
- Create: `tauri-desktop/react-ui/src/features/inputSource/hooks/useInputSourceState.test.ts`

**Interfaces:**
- Consumes: Task 2 settings DTO and Task 4 settings/history commands.
- Produces: enabled setting state and explicit reset-history action in SettingsPage. Feature remains disabled until user toggles it on.

- [ ] **Step 1: hook/API テストを書く** — initial disabled, enable persistence, disable stops new records while retaining history, clear history calls native command。
- [ ] **Step 2: React test を実行して失敗を確認する** — `cd tauri-desktop/react-ui && pnpm test -- useInputSourceState`
- [ ] **Step 3: disabled placeholder を有効な toggle に置き換え、履歴リセット action を追加する** — 既存 SettingsPage pending/error pattern を使う。
- [ ] **Step 4: locale 3 pack に学習設定・リセット確認・結果文言を追加する** — key parity を保つ。
- [ ] **Step 5: hook/UI 関連テストを再実行して通過を確認する** — `cd tauri-desktop/react-ui && pnpm test -- useInputSourceState`

### Task 7: 全体の品質確認

**Files:**
- Test/verify: Rust / React UI feature suites and project quality gates.

- [ ] **Step 1: Rust format を確認する** — `cd tauri-desktop/rust-native && cargo fmt -- --check`
- [ ] **Step 2: Rust lint と tests を実行する** — `cd tauri-desktop/rust-native && cargo clippy --all-targets -- -D warnings && cargo test`
- [ ] **Step 3: Frontend fix / typecheck / tests を実行する** — `cd tauri-desktop/react-ui && pnpm fix && pnpm typecheck && pnpm test`
- [ ] **Step 4: 手動 acceptance と切替速度を確認する** — 無効時に入力開始観測・記録なし、focus-onlyで記録なし、50/40境界、次回focusで自動切替完了後かつ入力前に提示、既存ルール抑止、承認登録、却下後再提示なし、履歴リセット、入力内容が永続化されないことを確認する。学習無効/有効の同一フォーカス切替を各30回以上測定し、p50/p95 と現在の `prepare_micros` / `source_timing` を比較する。学習有効時の切替 p95 が無効時より 2 ms または 5%（大きい方）を超えて悪化したら、切替 worker に学習処理が残っていないか調査して修正する。
