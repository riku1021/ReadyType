# 入力ソース切替のレイテンシ対策（施策メモ）

挙動の正（デフォルト）はフラグ未設定時の現行実装。
以下の高速化候補は **`READYTYPE_PROTO_*=1` で opt-in 計測**し、良ければデフォルト昇格する（Wave 1 は prototype、AX/tray は spec_only）。
状態表・昇格手順の正本: [input-source-switch-test-plan.md](./input-source-switch-test-plan.md)。

関連:

- ドメインモデル: [rules/ja-JP/input-source-model.md](./rules/ja-JP/input-source-model.md)
- 手動確認: [input-source-switcher-manual-test.md](./input-source-switcher-manual-test.md)
- テスト計画: [input-source-switch-test-plan.md](./input-source-switch-test-plan.md)

残しているもの（現行・implemented）:

- 詳細な切替 reason / detail、follow-up reselect の debug ログ
- `tmp/input-source-logs/` への JSONL ファイル出力（§11.0）
- 区間計測の縮小版（`processing` / `prepare` / `current` / `select` micros）

---

## 1. クリティカルパス（当時の計測モデル）

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

| 区間 | 主な対策（候補） |
|------|----------|
| AX | 2 パス指紋・enrich・構造打ち切り・高コスト属性の深さ制限・**属性一括取得**・**messaging timeout** |
| キュー | フォーカス合流（最新のみ処理） |
| handoff / current | ロック外 TIS・current キャッシュ・起動時 sync warm・**TIS 変更通知で invalidate** |
| select | 同一ソース省略 |
| publish / tray | 意味差分スキップ・差分更新・非同期 latest-wins |

---

## 2. Accessibility（フォーカス監視）— 候補

当時の主なファイル（再適用時の手がかり）:

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

## 3. Current 入力ソースキャッシュ — 候補

当時のファイル案: `application/current_source_cache.rs`

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

### 3.1 Manual override（フォーカス文脈ラッチ）— 候補

当時の置き場: `application/input_source_switcher.rs`

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

## 4. Switcher ワーカーとロック — 候補

当時の置き場: `application/switcher_controller.rs`

- **専用 worker** — フォーカスイベントをチャネルで受け、判定・切替をメイン UI から分離。（ベースにも worker はある）
- **`drain_queued_focus`** — 後続 Focus があれば最新スナップショットに置換（中間破棄）。バースト時の中間切替コストを削減。
- **ロック外 handoff** — `prepare_focus` は switcher mutex 内 → gateway/cache を取り出して **mutex 解放後**に `switch_input_source`。TIS 待ち中に switcher ロックを握らない。
- **NeedsSwitch 時の publish 遅延** — 切替完了後に state/tray publish をまとめ、切替前の tray 待ちを避ける。
- **`mutate_switcher` / `dispatch_publish`** — 変更はロック内、publish はガード drop 後（再入防止）。

---

## 5. Publish スキップ（`PublishFingerprint`）— 候補

当時の置き場: `application/input_source_switcher.rs`

tray / FE 向けに「見た目が変わらない」publish を省略する。

指紋に含める（**version / logs は除外**）:

`enabled`, `snoozed_until_unix_ms`, `current_app`, `matched_rule_id`, `current_input_source`, `target_input_source`, `last_decision`

- logs のみの更新は省略（診断ログは `list_switch_logs` で取得）。
- publish は mutex 解放後に実行（port / composition の約束）。

---

## 6. トレイ更新 — 候補

当時の置き場: `infrastructure/desktop/tray.rs`

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

| 施策 | 状態 | 効果 |
|------|------|------|
| `current == target` で select 省略 | **候補（未適用）** | ネイティブ再選択コスト回避。ベースは同ソースでも reselect |
| `AlreadySelected` | 現行あり | 決定ログ上の扱い |
| select 失敗時 1 回だけ `list(true)` + 再試行 | 現行あり | 欠落ソースの回復 |
| `handled_generation` | 現行あり | 同一 generation の二重処理を Ignored |
| postSelect / layoutOverride を reason に | 現行あり（ログ強化） | 切替直後の検証メモ |

### 7.1 当面非採用（デフォルト）

**アイドル後の「二段 select」（常時の二重撃ち）はデフォルト採用しない。**
速さ・AC-005（不要な再切替抑止）と逆行するため。診断で select 取りこぼしが再現したときだけ別チケット／PROTO 実験にする。
（macOS TIS 層の follow-up reselect はゲートウェイ実装の局所対策として現行あり。§7.2 参照。）

### 7.2 外部参考: InputSourceSwitcher から借りうるもの

参考実装: [InputSourceSwitcher](https://github.com/rn10/InputSourceSwitcher)（README.jp）。
相手は **CGEventTap で ^Space 相当を横取りして TIS 直切替**する用途で、ReadyType（AX フォーカス → ルール → TIS）とは主経路が異なる。**タップ／ショートカット横取りは借りない。**

すでに近いもの:

| 相手 | ReadyType（現行） |
|------|-------------------|
| TIS 直 select | `tis` / `switch_input_source` |
| 二段撃ち（再 select） | follow-up reselect +40 / +120 / +250ms（`READYTYPE_ABLATION_NO_FOLLOWUP` で切れる） |

あと借り（または点検）しうるのは次の 2 点。

#### 7.2.1 二段撃ちの詰め方（PROTO 実験向き）

**相手の意図:** select 直後にもう一度同じソースを選び、メニューだけ先に変わる／IME（ATOK 等）の活性化取りこぼしを防ぐ。メニューバーの一瞬チラつきは許容。

**ReadyType との差:** 後追い再 select 自体はある。違うのは **いつ・何回・どれだけ積極的か**。§7.1 の「常時アイドル後二重撃ちをデフォルトにしない」は維持し、効きが必要なら **`READYTYPE_PROTO_*` で opt-in 計測**する。

| 案 | 内容 | 狙い |
|----|------|------|
| A. 即時二段 | 1 回目 select の直後（ほぼ 0ms）にもう 1 回 | 相手に近い。活性化取りこぼしを最短で潰す |
| B. アイドル後だけ厚く | 最後の select から N 秒空いたときだけ、即時＋既存 follow-up | 「冷えたあと」だけ厚くする |
| C. タイミング調整 | 40/120/250 をずらす・本数を変える | いまの枠のまま効きを測る |

トレードオフ:

- **良い:** 偽切替（メニューだけ変わる）や IME 活性化遅れに効く可能性（AC-2）
- **悪い:** select 回数↑、チラつき、`selectMicros` 悪化、不要な再 select 感
- **正しさ:** 手動オーバーライド直後の自前 reselect は手動尊重と衝突しうる → P3（manual override latch）とセットで見る

測るとき: Lab + 手動で AC-2 / AC-3 と `selectMicros` p50。正本の昇格手順は [input-source-switch-test-plan.md](./input-source-switch-test-plan.md)。

#### 7.2.2 重い処理をコールバック上でやらない（メインキューへ渡す）

**相手の意図:** `CGEventTap` コールバック内では切替本体をやらす、すぐ return → 実処理はメインキューへ。タップ内で長引くと macOS がタップを無効化（`tapDisabledByTimeout`）し、取りこぼしが再発するため。待機も `usleep` ではなく後追い実行。

**ReadyType の対応関係（ショートカットタップは無いが同じ原則）:**

- TIS はメインキュー前提（macOS 26+ 等）
- フォーカス worker は switcher ロックを握ったまま TIS を待たない（ロック外 handoff）
- follow-up は呼び出しをブロックする sleep ではなく遅延スケジュール

新規機能というより **点検・維持チェックリスト**:

1. select 経路に同期 sleep（メイン／worker を止める待ち）が残っていないか
2. AX / フォーカスコールバック内で重い処理をしていないか（すぐキューに投げているか）
3. follow-up がメインを長時間占有していないか（短い select 連打は可、長い処理は NG）

崩れると相手の「タップ無効化」に相当し、監視停止・切替が突然効かない、が出うる。

| 項目 | 扱い |
|------|------|
| 二段撃ち強化（A/B/C） | 実験するなら PROTO。第一候補はここ |
| メインキュー／非ブロック | 点検・既存方針の厳守（フラグ化は不要） |

---

## 8. 診断メトリクス — 候補（当時）

高速化試行時に `SwitchLogEntry` へ載せた timing 列と診断 UI。

**現行（縮小版・適用済み）:** `processing_micros` / `prepare_micros` / `current_micros` / `select_micros`。手順は [input-source-switch-test-plan.md](./input-source-switch-test-plan.md)。

未適用の細分候補:

| フィールド | 測るもの |
|------------|----------|
| `ax_micros` | AX スナップショット（pipeline 外） |
| `queue_micros` | AX 完了 → `prepare_focus` 開始 |
| `handoff_micros` | NeedsSwitch 後〜 gateway/cache ロック取得（Done は 0） |
| `current_cache_hit` | current キャッシュ成否 |
| `finalize_micros` | complete_focus / 早期 push_log 直前まで |
| `pipeline_micros` | AX 完了〜ログまで（キュー・gateway 含む） |
| `publish_micros` | state/tray publish（pipeline 外。後書き） |

再適用時の読み方の目安:

- **AX が長い** → アプリの AX ツリー / Description 系ルール / debug キャプチャの有無
- **handoff が長い** → gateway ロック競合・warm 未実施
- **current miss が長い** → TIS `current()` スパイク（キャッシュ TTL・warm の効果確認）
- **select が長い** → OS 側の切替コスト（アプリ側ではこれ以上削りにくい）
- **publish が長い** → tray/IPC。Fingerprint / tray skip が効いているか

---

## 9. 意図的な二重スキップ一覧 — 候補

| 層 | 何が二重か | なぜ残すか |
|----|------------|------------|
| AX | `peek_focus_unchanged` と `commit_focus_change` | 早期 enrich 省略 + generation 冪等 |
| Publish / Tray | `PublishFingerprint` と `should_skip_tray_update` | IPC とメニュー更新を別層で抑止 |
| Select | `switch_input_source` 早期 return と `AlreadySelected` | ネイティブ省略 + 決定ログ |

---

## 10. 関連テスト（索引・当時）

再適用時に復活させ得るテストの手がかり。現行ツリーには無いものが多い。

| 領域 | テストの例 |
|------|------------|
| current キャッシュ | `current_source_cache::tests::*`、`current_source_cache_hits_within_ttl` |
| publish 省略 | `duplicate_already_selected_skips_redundant_publish` |
| select 省略 | `same_source_skips_select` |
| フォーカス合流 | `drain_queued_focus_keeps_latest_snapshot` |
| 指紋 / 属性計画 | `focus_monitor::tests::*` |
| tray skip | `tray_update_skips_identical_last_or_desired` |
| 診断 UI | `react-ui/tests/.../diagnosticsViewModel.test.ts` |

---

## 11. 診断ログ

### 11.0 ファイル出力（**現行あり**）

アプリ起動時から、診断 UI と同じ切替イベントをリポジトリ直下へ JSONL 追記する（**Git 対象外**）。

```text
tmp/input-source-logs/
  current.jsonl              ← いま読むのはこれ（1 行 = 1 JSON）
  sessions/<unixSecs>.jsonl  ← 過去起動分（最大 30）
```

- 1 行目は `kind=session_start`、以降は `kind=switch`
- フィールドは camelCase（`decision` / `reason` / `detail` / `processingMicros` / `prepareMicros` / `currentMicros` / `selectMicros` など）
- 上書きディレクトリ: 環境変数 `READYTYPE_INPUT_SOURCE_LOG_DIR`

### 11.1 reason / detail の見方

| 項目 | 状態 | 用途 |
|------|------|------|
| `reason` | 現行 | 判定理由。`selected target` / 失敗メッセージなど |
| `postSelect=match\|mismatch` | 現行 | `TISSelect` 直後の `current` 再読取 |
| `layoutOverride=` | 現行 | `TISCopyInputMethodKeyboardLayoutOverride`（あれば） |
| `enabled=` / `selectCapable=` | 現行（本番のみ） | select 失敗時の補助。ユニットテストでは OS 照会しない |
| `from`/`to` + `[imeLatin]` | 現行 | IME 英数モードとレイアウトの切り分け |
| follow-up reselect ログ | 現行（`debug_logging` 時） | +40/120/250ms の status / current |
| `focusKey` / override / coalesced | **候補（未適用）** | manual override・キュー合流向け |

---

## 12. 再適用時の実機確認（受け入れ案）

自動テストでは AX/TIS の体感は測れない。施策を戻したら診断で次を確認する。

1. **AX** — Cursor / Terminal / ブラウザでフォーカス移動を各数回。`ax_micros` の p50・外れ値が悪化していないこと。
2. **current 鮮度** — メニューバーで手動 IME 変更 → すぐにフィールドへフォーカス。誤った `AlreadySelected` やキャッシュ stale が減ること。
3. **ハング耐性** — 重い AX アプリでもパイプラインが長時間固まらないこと（timeout の効果）。
4. **回帰** — 自動切替の正しさが `57d310a` 相当から壊れていないこと（本メモの施策を戻す主因だったため）。
