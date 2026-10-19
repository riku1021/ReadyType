# Chromium / Electron の AX ツリー有効化（macOS）

フォーカス監視が **Google Chrome では取れ、Orca 等の Electron では取れない**／Quick Set が「読み取れません」になるときの背景と、ReadyType の打ち手。

関連:

- レイテンシ・空フォーカス非 sink: [input-source-switch-latency.md](./input-source-switch-latency.md)
- Accessibility 許可: [rules/ja-JP/input-source-model.md](./rules/ja-JP/input-source-model.md)

## 1. 症状（観測）

1. Chrome にフォーカス → 診断ログに `switch` が残る → Quick Set 成功
2. Orca（`com.stablyai.orca`）にフォーカス → **ログに Orca の `switch` が無い**
3. Quick Set を押すと stderr 例:
   - `focus snapshot is for com.google.Chrome but frontmost is com.stablyai.orca; …`
   - または起動直後の `no focused input field`

意味: **前面は Orca だが `last_snapshot` が更新されていない**（古い Chrome のまま、または `None`）。Quick Set のバグではなく、フォーカス監視が Orca の AX を sink できていない。

空の `AXFocusedUIElement` は意図的に sink しない（一時的な欠落で `last_snapshot` を潰さないため）。そのためツリー未起動のままではキャッシュが前アプリのまま残る。

## 2. 原因

Orca は **Electron（Chromium）**。Chromium 系は性能のため、支援技術を検知するまで **Web コンテンツの AX ツリーを遅延構築**する。

- ツリーがオフ → `AXFocusedUIElement` が空 → ReadyType は sink しない → `last_snapshot` 更新なし
- 過去に VoiceOver 等で一度起きていたプロセスでは、同じ Orca でもログに `xterm-helper-textarea` 等が残り「うまくいく」ことがある（間欠の説明）

参照:

- [Chromium Accessibility（AT 検知まで遅延）](https://www.chromium.org/developers/design-documents/accessibility/)
- [Electron Accessibility（第三者は `AXManualAccessibility`）](https://www.electronjs.org/docs/latest/tutorial/accessibility)

## 3. なぜ `AXEnhancedUserInterface` だけにしないか

「Manual がダメなら Enhanced にするなら、最初から Enhanced でよいのでは？」→ **違う。**

| 属性 | 役割 |
|------|------|
| `AXManualAccessibility` | Electron が **第三者 AT 向けに追加**した opt-in。VoiceOver 用フラグを避けて a11y だけ起こす |
| `AXEnhancedUserInterface` | もともと **VoiceOver 用**。Chromium もこれを見て a11y をオンにするが、**ウィンドウ移動・リサイズが遅くなる／おかしくなる**副作用が知られている |

Electron が Manual を足した理由（要約）: `AXEnhancedUserInterface` は VoiceOver 予約であり、第三者やウィンドウマネージャとの干渉がある。別属性で a11y だけ有効化できるようにした（[electron#10305](https://github.com/electron/electron/pull/10305)）。Enhanced の副作用・トグル遅延は [Chromium issue 40865608](https://issuetracker.google.com/issues/40865608)、Firefox 側でも同趣旨の議論（[bug 1664992](https://bugzilla.mozilla.org/show_bug.cgi?id=1664992)）がある。

したがって ReadyType は次の順にする（cua 等も同じ方針）:

1. **先に** `AXManualAccessibility = true`（使えるなら副作用の少ない方）
2. 失敗（未対応など）なら **`AXEnhancedUserInterface = true`**（古い Electron / Chrome 等のフォールバック）

「同じことを二度やる」のではなく、**使える属性を優先し、無いアプリだけ旧フラグに落とす**。最初から Enhanced だけにすると、Manual が効くアプリでも不要な副作用リスクを負う。

アプリによってはどちらか一方しか見ない、という実例もある。フォールバック（または両方試す）は互換のため。

## 4. ReadyType の実装

| 場所 | 内容 |
|------|------|
| `infrastructure/platform/macos/accessibility.rs` | `request_chromium_accessibility` — Manual → Enhanced |
| `infrastructure/platform/macos/focus_monitor.rs` | アプリ activate 時に上記を実行。成功した PID は初回のみ約 **500ms** settle（ツリー構築が非同期のため）。空スナップショットかつ未 settle なら一度再試行 |

ネイティブ Cocoa アプリは両属性とも `AttributeUnsupported` になり無視される想定。settle は PID 単位で一度だけ（繰り返しの 500ms ペナルティを避ける）。

## 5. 確認の目安

Orca のターミナル等にフォーカスしたあと、`tmp/input-source-logs/current.jsonl` に `appId":"com.stablyai.orca"` の `switch` が出ること。その状態で Quick Set が前面アプリのセッションを開けること。
