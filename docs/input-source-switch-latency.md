# 入力ソース切替のレイテンシ対策

フォーカス変更から入力ソース切替・状態反映までのクリティカルパスを短くするための実装メモ。  
実装の正本は `tauri-desktop/rust-native`。診断 UI は `tauri-desktop/react-ui` の inputSourceDiagnostics。

関連:

- ドメインモデル: [rules/ja-JP/input-source-model.md](./rules/ja-JP/input-source-model.md)
- 手動確認: [input-source-switcher-manual-test.md](./input-source-switcher-manual-test.md)

---

## 1. クリティカルパス

```text
AX capture (ax_micros)
  → FocusEventSink（pipeline_started）
  → worker キュー待ち (queue_micros)
  → prepare_focus ルール判定 (prepare_micros)
  → [NeedsSwitch] gateway/cache ロック取得 (handoff_micros)
  → current() + キャッシュ (current_micros / current_cache_hit)
  → select() (select_micros)  ※同一ソースなら 0
  → complete_focus / push_log (finalize_micros)
  → publish state + tray (publish_micros)  ※ pipeline 外
```

| 区間 | 主な対策 |
|------|----------|
| AX | 2 パス指紋・enrich・構造打ち切り・高コスト属性の深さ制限・**属性一括取得**・**messaging timeout** |
| キュー | フォーカス合流（最新のみ処理） |
| handoff / current | ロック外 TIS・current キャッシュ・起動時 sync warm・**TIS 変更通知で invalidate** |
| select | 同一ソース省略 |
| publish / tray | 意味差分スキップ・差分更新・非同期 latest-wins |

---

## 2. Accessibility（フォーカス監視）

主要ファイル:

- `infrastructure/platform/focus_monitor.rs` — 定数・指紋・`AttributePlan`・`MonitorState`
- `infrastructure/platform/macos/accessibility.rs` — 親ウォーク / enrich
- `infrastructure/platform/macos/focus_monitor.rs` — 2 パス評価
- `application/switcher_controller.rs` — `switching_attribute_index`（アプリ別 lean 属性）

### 2.1 定数

| 記号 | 値 | 意図 |
|------|-----|------|
| `MAX_PARENT_DEPTH` | `14` | 親チェーン上限 |
| `EXPENSIVE_ATTRIBUTE_MAX_DEPTH` | `0` | Description / Title 等は focused のみ（祖先で長い Description を読むと AX が跳ねる） |
| `FINGERPRINT_IDENTITY_DEPTH` | `6` | 第 1 パスで祖先 Dom* / Identifier を読む深さ上限（**identity key 自体は focused のみ**） |
| `FALLBACK_INTERVAL_MS` | `50` | Observer 未対応時のポーリング |
| `FOCUSED_PROBE_TIMEOUT_SECONDS` | `0.05` | `AXFocusedUIElement` 取得専用。空フォーカスで長く待たない |
| `MESSAGING_TIMEOUT_SECONDS` | `0.15` | 親ウォーク / enrich の `AXUIElementSetMessagingTimeout`（失敗は無視） |

### 2.2 Role 候補判定付き指紋ウォーク（通常パス）

debug キャプチャ以外:

1. **focused のみ** fingerprint（Role / Identifier / DomIdentifier / DomClassList）。FOCUSED は 50ms timeout。
2. **`is_switch_candidate_role`** — 非候補（`AXButton` / `AXLink` / `AXRow` 等）は focused のみ・**enrich なし**で sink。
3. **候補時** — shallow の focused ノードを再利用し **`extend_parent_walk`** で親だけ伸ばす（`AXFocusedUIElement` の再 IPC なし）→ `peek_focus_unchanged` → 変化時のみ enrich。`role_may_need_description` が false なら Description 等高コスト属性は enrich しない（`AXGroup` の Composer は DomClass で足りる）。
4. **`commit_focus_change`** — generation を進め sink。**`empty` は commit/sink しない**（一時 FOCUSED 失敗で generation を進めない）。

候補 Role: `AXTextField` / `AXTextArea` / `AXComboBox` / `AXSearchField` / `AXGroup` / `AXWebArea`、または Role に `text` を含む（`AXStaticText` 含む）。

### 2.3 その他の AX 削減

- **構造打ち切り** — `AXWindow` / `AXApplication` で親辿り停止。
- **高コスト属性ガード** — Description / RoleDescription / Title / Help / Placeholder は `depth > EXPENSIVE_ATTRIBUTE_MAX_DEPTH` なら読まない。
- **lean attribute index** — 有効ルールの条件属性だけをアプリ単位で集める（`plan_for_app`）。
- **pending PID 合流** — 同一 pid の重複 Evaluate を coalesce。
- **指紋から Description 除外** — 揮発テキストによる誤再評価を防ぐ。
- **focus identity は focused のみ** — Role / DomIdentifier / 正規化 DomClassList / Identifier。祖先は identity に使わない（チェーン長・`visible` 等の揺れで generation が進むのを防ぐ）。ルールマッチは従来どおり親チェーンを enrich。
- **属性一括取得** — ノードごとに `AXUIElementCopyMultipleAttributeValues`（`copy_attribute_texts`）。単発 IPC をまとめ、失敗時のみ単発へフォールバック。
- **二段 timeout** — FOCUSED は 50ms、親ウォーク / enrich は 150ms。

### 2.4 トレードオフ

| 選択 | 得るもの | 失うもの |
|------|----------|----------|
| `EXPENSIVE_ATTRIBUTE_MAX_DEPTH=0` | AX スパイク抑制 | 祖先の Description マッチ不可（ルールは focused / Dom* 前提） |
| 指紋に Description 非含有 | 不要な再評価減 | Description だけ変わっても再切替しない |
| identity は focused のみ | 入力中の祖先揺れ・チェーン長変化で再切替しない | focused の属性が同一の別フィールドは同一扱い |
| empty 非 commit | 空フォーカス blip で generation が進まない | FOCUSED 失敗中はフォーカス喪失をログに出さない |
| debug フル属性 1 パス | 診断用の完全 AX | 速度最適化パスをバイパス |
| FOCUSED probe 50ms | `elements=0` の 150ms 級待ちを短縮 | 極端に遅い FOCUSED は一度空になり、次の通知で再試行 |
| messaging timeout 0.15s（親/enrich） | 外れ値ハングの上限 | 極端に遅い AX は途中失敗し得る（フォールバック/欠落） |
| 非候補 Role は浅い AX | Button/Link 等の無駄な親ウォーク削減 | `AXButton`+DomClass のようなルールはマッチしない（IME 用途では許容） |
| 候補は FOCUSED 再利用 | 候補フルパスの二重 FOCUSED IPC 削減 | — |

---

## 3. Current 入力ソースキャッシュ

ファイル: `application/current_source_cache.rs`

| 項目 | 内容 |
|------|------|
| 型 | `CurrentInputSourceCache` |
| TTL | `DEFAULT_TTL` = **2500ms** |
| 用途 | `gateway.current()` の短時間キャッシュ（TIS の間欠スパイク抑制） |

挙動:

- **get** — TTL 内なら OS を呼ばない。
- **select 成功後** — 新ターゲットで `put`。
- **select 失敗時** — `invalidate`。
- **warm（sync）** — モニタ開始時、呼び出しスレッドで `current()` して温める。バックグラウンド warm だと最初の NeedsSwitch と gateway ロックが競合し、handoff が 100ms 超になり得る（TIS は main へ sync）。
- **TIS 変更通知** — `kTISNotifySelectedKeyboardInputSourceChanged`（CF distributed）を購読し、コールバックで `invalidate` + **manual override ラッチ**（後述）。

関連ファイル:

- port: `application/ports/current_input_source_change.rs`
- macOS: `infrastructure/platform/macos/current_input_source_change.rs`
- 配線: `SwitcherController::new` で cache / switcher を sink に閉じる

トレードオフ: TTL は通知取りこぼしの安全網として残す。自前 `select` でも通知が飛び得るが、select 成功後の `put` と競合しても余分な `current()` 1 回程度。

### 3.1 Manual override（フォーカス文脈ラッチ）

ファイル: `application/input_source_switcher.rs`

xxkb 系と同様、**同一フォーカス文脈では手動 IME 切替をルール再適用で上書きしない**。

| 状態 | 意味 |
|------|------|
| `current_focus_key` | 直近処理した正規化 fingerprint（`FocusSnapshot::focus_key`） |
| `manual_override_key` | 手動変更を尊重する文脈 |
| `last_auto_target` | ReadyType が最後に選んだソース |
| `suppress_tis_as_manual` | 自前 `select` 中の TIS を手動扱いしない |

挙動:

1. 外部 TIS 変更（suppress 中でなく、`current != last_auto_target`）→ `manual_override_key = current_focus_key`。
2. `prepare_focus` で `focus_key` が変わったら override クリア。同じ key で override 中なら **select スキップ**（ログ理由で明示）。
3. アプリ `activate` では override / focus_key をリセット（再前面化で再評価）。

---

## 4. Switcher ワーカーとロック

ファイル: `application/switcher_controller.rs`

- **専用 worker** — フォーカスイベントをチャネルで受け、判定・切替をメイン UI から分離。
- **`drain_queued_focus`** — 後続 Focus があれば最新スナップショットに置換（中間破棄）。バースト時の中間切替コストを削減。
- **ロック外 handoff** — `prepare_focus` は switcher mutex 内 → gateway/cache を取り出して **mutex 解放後**に `switch_input_source`。TIS 待ち中に switcher ロックを握らない。
- **NeedsSwitch 時の publish 遅延** — 切替完了後に state/tray publish をまとめ、切替前の tray 待ちを避ける。
- **`mutate_switcher` / `dispatch_publish`** — 変更はロック内、publish はガード drop 後（再入防止）。

---

## 5. Publish スキップ（`PublishFingerprint`）

ファイル: `application/input_source_switcher.rs`

tray / FE 向けに「見た目が変わらない」publish を省略する。

指紋に含める（**version / logs は除外**）:

`enabled`, `snoozed_until_unix_ms`, `current_app`, `matched_rule_id`, `current_input_source`, `target_input_source`, `last_decision`

- logs のみの更新は省略（診断ログは `list_switch_logs` で取得）。
- publish は mutex 解放後に実行（port / composition の約束）。

---

## 6. トレイ更新

ファイル: `infrastructure/desktop/tray.rs`

| 施策 | 内容 |
|------|------|
| A. 差分更新 | MenuItem ハンドル保持。変更フィールドだけ `set_text` / `set_enabled`（フル `set_menu` しない） |
| B. 非同期 latest-wins | `readytype-tray` スレッドで適用。古い generation は破棄 |
| スキップ | `should_skip_tray_update` — `last == next` または `desired == next` ならスケジュールしない |
| デッドロック回避 | `MenuItem::set_*` は main へ marshal する。`run_on_main_thread` 内で `set_*` するとデッドロックするため、専用スレッドから呼ぶ |
| ワーカー非待機 | `update` はスケジュールして即 return |

`PublishFingerprint` と tray skip は層が違う（状態 publish vs メニュー更新）。意図的な二重防御。

---

## 7. Select パス

ファイル: `application/input_source_switcher.rs`

| 施策 | 効果 |
|------|------|
| `current == target` で select 省略 | `select_micros = 0`、ネイティブ再選択コスト回避 |
| `AlreadySelected` | 決定ログ上もスキップとして記録 |
| select 失敗時 1 回だけ `list(true)` + 再試行 | 欠落ソースの回復 |
| `handled_generation` | 同一 generation の二重処理を Ignored |

### 7.1 当面非採用

**アイドル後の「二段 select」（常時の二重撃ち）は採用しない。**  
速さ・AC-005（不要な再切替抑止）と逆行するため。診断で select 取りこぼしが再現したときだけ別チケットにする。  
（macOS TIS 層に Electron 向け follow-up reselect がある場合は、ゲートウェイ実装の局所対策として扱う。）

---

## 8. 診断メトリクス

ログ型: `SwitchLogEntry`（`application/switcher_state.rs`）  
UI: `features/inputSourceDiagnostics/`（`diagnosticsViewModel.ts` / `DiagnosticsPage`）

| フィールド | 測るもの |
|------------|----------|
| `ax_micros` | AX スナップショット（pipeline 外） |
| `queue_micros` | AX 完了 → `prepare_focus` 開始 |
| `prepare_micros` | ルール判定 |
| `handoff_micros` | NeedsSwitch 後〜 gateway/cache ロック取得（Done は 0） |
| `current_micros` / `current_cache_hit` | `gateway.current()` とキャッシュ成否 |
| `select_micros` | `gateway.select()`（未実行 0。失敗時の list+再試行含む） |
| `finalize_micros` | complete_focus / 早期 push_log 直前まで |
| `processing_micros` | 判定処理全体 |
| `pipeline_micros` | AX 完了〜ログまで（キュー・gateway 含む） |
| `publish_micros` | state/tray publish（pipeline 外。後書き） |

UI では p50 系集計・hit/miss 分割・外れ値ハイライト（`findLatencyOutliers`）を表示する。ログリング上限は 200。

読み方の目安:

- **AX が長い** → アプリの AX ツリー / Description 系ルール / debug キャプチャの有無
- **handoff が長い** → gateway ロック競合・warm 未実施
- **current miss が長い** → TIS `current()` スパイク（キャッシュ TTL・warm の効果確認）
- **select が長い** → OS 側の切替コスト（アプリ側ではこれ以上削りにくい）
- **publish が長い** → tray/IPC。Fingerprint / tray skip が効いているか

---

## 9. 意図的な二重スキップ一覧

| 層 | 何が二重か | なぜ残すか |
|----|------------|------------|
| AX | `peek_focus_unchanged` と `commit_focus_change` | 早期 enrich 省略 + generation 冪等 |
| Publish / Tray | `PublishFingerprint` と `should_skip_tray_update` | IPC とメニュー更新を別層で抑止 |
| Select | `switch_input_source` 早期 return と `AlreadySelected` | ネイティブ省略 + 決定ログ |

---

## 10. 関連テスト（索引）

| 領域 | テストの例 |
|------|------------|
| current キャッシュ | `current_source_cache::tests::*`（`invalidate_clears_cached_value` 含む）, `current_source_cache_hits_within_ttl` |
| publish 省略 | `duplicate_already_selected_skips_redundant_publish` |
| select 省略 | `same_source_skips_select` |
| フォーカス合流 | `drain_queued_focus_keeps_latest_snapshot` |
| 指紋 / 属性計画 | `focus_monitor::tests::*`（`attributes_beyond_fingerprint_*` など） |
| tray skip | `tray_update_skips_identical_last_or_desired` |
| 診断 UI | `react-ui/tests/.../diagnosticsViewModel.test.ts` |

---

## 11. 実機での確認手順（受け入れ）

自動テストでは AX/TIS の体感は測れない。診断画面で次を確認する。

1. **AX** — Cursor / Terminal / ブラウザでフォーカス移動を各数回。`ax_micros` の p50・外れ値が悪化していないこと（一括取得後は同等以上を期待）。
2. **current 鮮度** — メニューバーで手動 IME 変更 → すぐにフィールドへフォーカス。誤った `AlreadySelected` が減り、変更後の最初の lookup で `current_cache_hit=false` になりやすいこと。
3. **ハング耐性** — 重い AX アプリでもパイプラインが長時間固まらないこと（timeout の効果）。
