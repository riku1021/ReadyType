# 入力ソース切替テスト計画（正本）

関連:

- レイテンシ施策メモ: [input-source-switch-latency.md](./input-source-switch-latency.md)
- 手動 E2E マトリクス: [input-source-switcher-manual-test.md](./input-source-switcher-manual-test.md)

**デフォルト（フラグ未設定）が現行の正。** すべての `READYTYPE_PROTO_*` は未設定時 OFF。フラグ無し起動 = いま動いている挙動を担保する。

## 1. 目的

1. Lab 欄フォーカスで自動切替が正しいこと（AC-1）
2. メニューバー / `TISCopyCurrentKeyboardInputSource` だけの偽切替を、手入力なし（自動 1 キー）で検知できること（AC-2）
3. 自動切替後の手動 IME 切替が同一フォーカスで尊重され、そのソースで入力できること（AC-3）
4. 安定 / 高速の各対策が正しさを壊さず、過剰でもなく、速度に効いていること（AC-4 / AC-5）
5. latency.md 未適用候補をフラグ付き prototype として計測し、良ければデフォルト昇格する

人手で期待文を打つことは受け入れ条件に含めない。

## 2. 成功条件

| ID | 条件 |
|----|------|
| AC-1 | Lab 奇数欄 → preferred ja、偶数欄 → preferred en |
| AC-2 | settle 後 `current == target`、かつ自動 1 キーで欄の実入力が target 側挙動 |
| AC-3 | 同一フォーカスで手動切替後、ルール側へ戻さず、自動 1 キーが手動側 |
| AC-4 | 現行対策の ON/OFF で正しさスイートが壊れない。OFF で悪化するなら必要、正しさも速度も差なしなら削除候補 |
| AC-5 | 同一マシンでフル ON / フル OFF / 単体 OFF の p50・p95 を表比較できる |

## 3. 状態ラベルとフラグの役割

| status | 意味 |
|--------|------|
| `implemented` | デフォルト ON（現行） |
| `prototype` | コードあり・デフォルト OFF・`READYTYPE_PROTO_*=1` で計測可 |
| `spec_only` | 未再適用（AC のみ。Wave 2/3） |
| `rejected` | 計測の結果不採用 |

| フラグ種別 | 用途 |
|------------|------|
| `READYTYPE_ABLATION_NO_*` | **implemented** 対策を切って妥当性検証 |
| `READYTYPE_PROTO_*` | **prototype** 候補を載せて計測（opt-in） |

### 3.1 implemented（デフォルト ON）

| 対策 | 場所 | Ablation env |
|------|------|----------------|
| follow-up reselect +40/120/250ms | `tis.rs` | `READYTYPE_ABLATION_NO_FOLLOWUP=1` |
| 同一ソース reselect | `switch_input_source` | `READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT=1`（省略モード計測） |
| select 失敗時 list + 1 回 retry | `switch_input_source` | `READYTYPE_ABLATION_NO_RETRY=1` |
| focus fingerprint / handled_generation / pending PID coalesce | focus_monitor / switcher | （正しさテストのみ） |
| worker + ロック外 handoff | switcher_controller | — |
| Lab プリセット / `report_lab_focus` | presets / controller | — |
| 区間計測（縮小版） | SwitchLogEntry | — |

### 3.2 prototype（Wave 1・デフォルト OFF）

| ID | 対策 | env（ON で有効） | 測る指標 |
|----|------|------------------|----------|
| P1 | Current cache TTL 2500ms + put/invalidate + sync warm | `READYTYPE_PROTO_CURRENT_CACHE=1` | `currentMicros`、AC-2 |
| P2 | TIS 変更通知 → cache invalidate | `READYTYPE_PROTO_TIS_NOTIFY=1`（P1 推奨） | current 鮮度、AC-3 |
| P3 | Manual override latch | `READYTYPE_PROTO_MANUAL_OVERRIDE=1` | AC-3 |
| P4 | `drain_queued_focus` | `READYTYPE_PROTO_DRAIN_FOCUS=1` | processing / ready（バースト） |
| P5 | PublishFingerprint | `READYTYPE_PROTO_PUBLISH_FINGERPRINT=1` | publish 回数 |

### 3.3 spec_only（Wave 2/3・フラグ予約）

| 予約 env | 内容 |
|----------|------|
| `READYTYPE_PROTO_AX_FAST=1` | expensive depth / messaging timeout / batch / candidate-role 2パス等 |
| `READYTYPE_PROTO_TRAY_DIFF=1` | tray skip / 差分更新 / latest-wins |

## 4. 検証レイヤ

| レイヤ | 対象 | 実行 |
|--------|------|------|
| CI | ルール照合・合成 Focus・手動尊重・ablation / PROTO OFF・ON 正しさ・FE IPC | `cargo test` / `pnpm test`（フラグ未設定が回帰ゲート。PROTO ON は dedicated テスト） |
| macOS Lab スモーク | 窓・フォーカス・TIS・自動 1 キー（AppKit 窓が前面に出る。`cargo test` 不可） | `mise run rust-native:lab-smoke` |
| 手動マトリクス | ハーネス未整備時 | [input-source-switcher-manual-test.md](./input-source-switcher-manual-test.md) |

## 5. 区間計測フィールド

| フィールド | 意味 |
|------------|------|
| `processingMicros` | ログ push までの処理時間 |
| `prepareMicros` | ルール判定 |
| `currentMicros` | `gateway.current()`（キャッシュ hit 時は短い） |
| `selectMicros` | `gateway.select`（retry 含む） |

ハーネス側 `readyMicros` は Lab スモークが壁時計で測る。集計は p50 / p95。CI ゲートには使わない。

ファイル診断ログ（`tmp/input-source-logs/current.jsonl`）の `session_start` / `switch` には `flags.ablation` / `flags.proto` が入る。空配列 = 本番デフォルト。ON の slug 例: `noFollowup`, `currentCache`, `drainFocus`。

## 6. Ablation（implemented を切る）

```bash
mise run rust-native:lab-smoke
READYTYPE_ABLATION_NO_FOLLOWUP=1 mise run rust-native:lab-smoke
READYTYPE_ABLATION_NO_SAME_SOURCE_RESELECT=1 mise run rust-native:lab-smoke
READYTYPE_ABLATION_NO_RETRY=1 mise run rust-native:lab-smoke
```

| variant | 判定の目安 |
|---------|------------|
| baseline_full | 基準 |
| no_followup | AC-2 失敗率↑なら必要 |
| no_same_source_reselect | 偽切替再現なら reselect 維持 |
| no_retry | 欠落ソース時のみ悪化なら必要 |

## 7. Prototype マトリクス（Wave 1）

前提: 同一 Mac・同一 OS・preferred ja/en。**まずフラグ無し baseline が緑であること。**

```bash
# baseline（現行の正）
mise run rust-native:lab-smoke

# 単体
READYTYPE_PROTO_PUBLISH_FINGERPRINT=1 mise run rust-native:lab-smoke
READYTYPE_PROTO_DRAIN_FOCUS=1 mise run rust-native:lab-smoke
READYTYPE_PROTO_CURRENT_CACHE=1 mise run rust-native:lab-smoke
READYTYPE_PROTO_CURRENT_CACHE=1 READYTYPE_PROTO_TIS_NOTIFY=1 mise run rust-native:lab-smoke
READYTYPE_PROTO_MANUAL_OVERRIDE=1 mise run rust-native:lab-smoke

# Wave1 全部
READYTYPE_PROTO_CURRENT_CACHE=1 READYTYPE_PROTO_TIS_NOTIFY=1 \
  READYTYPE_PROTO_MANUAL_OVERRIDE=1 READYTYPE_PROTO_DRAIN_FOCUS=1 \
  READYTYPE_PROTO_PUBLISH_FINGERPRINT=1 \
  mise run rust-native:lab-smoke
```

記録テンプレ（`tmp/input-source-ablation/`）:

| variant | AC-2 fail | AC-3 fail | processing p50/p95 | current p50/p95 | ready p50/p95 | 判定 |
|---------|-----------|-----------|--------------------|-----------------|---------------|------|
| baseline | | | | | | 基準 |
| proto_p5 | | | | | | |
| proto_p4 | | | | | | |
| proto_p1 | | | | | | |
| proto_p1_p2 | | | | | | |
| proto_p3 | | | | | | |
| proto_wave1_all | | | | | | |

## 8. 昇格チェックリスト（prototype → implemented）

すべて満たしたらデフォルト ON を検討する:

1. フラグ無し回帰（既存 `cargo test` / FE）が緑のまま
2. PROTO ON で dedicated 正しさテストが緑
3. Lab スモークで AC-2 / AC-3 失敗率が baseline より悪化していない
4. 対象区間の p50 が同一マシンで意味のある改善（または安定性のみの候補なら失敗率改善）
5. 手動マトリクスの関連行を実施し記録
6. 正本の表で `prototype` → `implemented` に移し、PROTO フラグを廃止または常時相当に変更

## 9. Lab スモーク手順（概要）

1. アクセシビリティ許可・自動切替 ON・監視中・preferred ja/en
2. `mise run rust-native:lab-smoke`（必要なら PROTO env 付き）
3. 各欄フォーカス → settle → TIS `current` が目標ソース（Lab スモークの主判定）
4. 別ソースへ `TISSelect` → `current` 維持。IME で「あ」になることの証明は手動マトリクス（合成キーは Google 日本語等で不可靠）
5. 失敗時は `tmp/input-source-logs/current.jsonl`

## 10. スコープ外

- Playwright だけで Lab 窓・IME を証明
- 人手の期待文打鍵を必須化
- prototype を計測なしでデフォルト ON にすること
- アイドル後二段 select の**デフォルト常時採用**（latency.md §7.1。PROTO 実験は §7.2.1）
- InputSourceSwitcher 流の CGEventTap / ショートカット横取り（latency.md §7.2）
- 非 macOS
