# フォーカス監視は全アプリに対応できない

ReadyType の自動切替と Quick Set は、macOS Accessibility（AX）で **今フォーカスしている入力欄**を読む前提である。`AXManualAccessibility` / `AXEnhancedUserInterface` で Chromium 系のツリーを起こしても、**すべてのアプリで常に成功する、とは言えない。** 技術的にも、同じ手段ですべてをカバーすることはできない。

関連:

- Chromium / Electron の起こし方: [input-source-chromium-ax.md](./input-source-chromium-ax.md)
- 空フォーカスを sink しない理由: [input-source-switch-latency.md](./input-source-switch-latency.md)

## 1. 結論

| 範囲 | 見込み |
|------|--------|
| 標準 AppKit、AX を公開しているネイティブアプリ | 高い |
| Electron / Chromium で Manual か Enhanced に反応するもの（Orca など） | 今回の起こしで改善する。全ビルドを保証はしない |
| AX を公開していない、または別の起こし方が要るアプリ | ReadyType 側だけでは原理的に無理 |

「すべてのアプリ」は目標にしない。よく使うアプリで AX が取れる範囲を広げる、が上限である。

## 2. なぜ全対応は不可能に近いか

macOS に「どのアプリでも正しいフォーカス入力欄を必ず返す」共通 API はない。AX は **アプリが公開した要素だけ**を返す（[Accessibility Programming Guide](https://developer.apple.com/library/archive/documentation/Accessibility/Conceptual/AccessibilityMacOSX/OSXAXmodel.html)）。クライアントが無いツリーを作る正規手段はない。

公開していない典型:

- Metal / OpenGL / 独自 Canvas。OS から見ると要素が無く、フラグを立てても空のまま（[Warp の GPU ターミナル](https://github.com/warpdotdev/warp/issues/11160)）。ゲームやエミュレータも同型
- アクセシビリティはアプリ側・OS ポリシー側の両方が opt-in で、オフのときは「UI が無い」と区別しにくい（[xa11y: Accessibility API Quirks](https://xa11y.dev/explanation/accessibility-quirks/)）

## 3. 起こし方はフレームワークごとに違う

今回の実装は **アプリ要素に `AXManualAccessibility`、失敗時に `AXEnhancedUserInterface`** を一度立てる。これは Electron 公式の第三者向け手順と、古い Chromium 向けフォールバックである。次はカバーしない。

| 系統 | ギャップ |
|------|----------|
| Electron 28+ の webview | アプリ根だけでは足りず、**renderer の AX 要素にも Manual が要る**報告がある。renderer 再起動でフラグが消える（[tabby #19](https://github.com/FuJacob/tabby/issues/19)） |
| 古い Electron | `AXManualAccessibility` が `AttributeUnsupported` になる不具合があった（[electron#37465](https://github.com/electron/electron/issues/37465)、後に修正） |
| Chrome 本体 | 伝統的には VoiceOver 用の `AXEnhancedUserInterface` を見る。Manual は Electron 追加属性 |
| Firefox | Enhanced や「アプリの role を読んだら起こす」など別経路。Manual は Electron 向けの提案に留まった時期がある（[bug 1664992](https://bugzilla.mozilla.org/show_bug.cgi?id=1664992)、[bug 1845364](https://bugzilla.mozilla.org/show_bug.cgi?id=1845364)） |
| Flutter（macOS） | `AXEnhancedUserInterface` が立ってもネイティブツリーが空、外部ツールが `TextField` を見つけられない、という報告（[flutter#189683](https://github.com/flutter/flutter/issues/189683)、[flutter#191503](https://github.com/flutter/flutter/issues/191503)）。`AXManualAccessibility` は未対応 |
| Premiere 等の独自描画 | `AXFocusedUIElement` が nil のまま、という報告（[Hammerspoon #3433](https://github.com/Hammerspoon/hammerspoon/issues/3433)） |

一つの属性ですべての UI キットを起こす API はない。

## 4. 起きても不安定なまま残る点

- ツリー構築は非同期。settle（ReadyType は初回約 500ms）より遅いと、その瞬間は空
- 空の `AXFocusedUIElement` は sink しない。その間 `last_snapshot` は前アプリのままになり、Quick Set は「読み取れません」になる
- Accessibility の TCC 未許可では、どの起こしも効かない

## 5. 別手段でも「すべて」には届かない

| 手段 | 限界 |
|------|------|
| AX（現行） | アプリが公開した範囲だけ |
| OCR / 画面認識 | 入力欄の意味が曖昧。遅延と誤検知があり、入力ソース切替の根拠に弱い |
| 座標とキー送信 | フォーカスの保証が無く、アプリごとに壊れやすい |
| アプリごとの専用連携 | 全アプリを網羅できない |

いずれも「全アプリで、常に正しい入力欄」までは届かない。

## 6. ReadyType が今やっていること

対応しているのは、だいたい次である。

- AX を最初から出すネイティブ Cocoa（起こし属性は `AttributeUnsupported` で無視）
- Manual か Enhanced に反応する Chromium / Electron（[input-source-chromium-ax.md](./input-source-chromium-ax.md)）

やっていないこと: renderer への再適用、Firefox / Flutter 専用の起こし、AX が無いアプリ向けの OCR やキー送信フォールバック。
