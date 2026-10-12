<p align="center">
  <img src="tauri-desktop/react-ui/public/pngs/logo.png" alt="ReadyType" width="96" />
</p>

<h1 align="center">ReadyType</h1>

<p align="center"><strong>入力する場所に、適切な言語を。</strong></p>

<p align="center">
  入力欄に応じて入力ソースを自動で切り替えるデスクトップアプリ。<br />
  現在は <strong>macOS</strong> 対応（Windows / Linux は開発予定）。
</p>

<p align="center">
  <img alt="macOS" src="https://img.shields.io/badge/platform-macOS-black" />
  <img alt="Tauri 2" src="https://img.shields.io/badge/Tauri-2-24C8DB" />
  <img alt="React" src="https://img.shields.io/badge/React-19-61DAFB" />
  <img alt="Rust" src="https://img.shields.io/badge/Rust-1.98-DEA584" />
</p>

---

## Overview｜概要

ReadyType は、フォーカスした入力欄のルールに従って **日本語・英語などの入力ソースを自動切替** します。

同じアプリ内でも、チャットは日本語・検索欄は英語のように、**入力欄単位**で設定できます。macOS の Accessibility API と Text Input Source を利用します。

> **プライバシー:** 入力した文章は読み取りません。入力欄の識別に必要な情報のみを使い、設定は端末内に保存します。

---

## Motivation｜開発背景

日本語を打ちたいのに英語のまま、英語を打ちたいのに日本語変換が始まる——そんな日常の小さなストレスから生まれました。

アプリ単位の切替では足りず、**同じアプリの中でも欄ごとに言語が違う**場面が多いことに気づき、入力欄単位の自動切替を目指して開発しています。

第一号のユーザーは、私自身です。

---

## Features｜主な機能

| 機能 | 内容 |
| --- | --- |
| **入力欄単位の自動切替** | 登録ルールに従い、フォーカス時に最適な入力ソースへ切替 |
| **アプリ・ルール管理** | アプリ一覧から対象を選び、入力欄ごとのルールを編集 |
| **Quick Set** | 入力欄にフォーカスしたままショートカットでルールを登録 |
| **オンボーディング** | Accessibility 権限や初期設定を初回に案内 |

---

## Demo｜デモ

<p align="center">
  <a href="https://youtu.be/dPCCXdxS-bg">
    <img
      src="https://img.youtube.com/vi/dPCCXdxS-bg/maxresdefault.jpg"
      alt="ReadyType デモ動画を再生"
      width="720"
    />
  </a>
</p>

<p align="center">
  <a href="https://youtu.be/dPCCXdxS-bg">▶ YouTube でデモを見る</a>
  （限定公開）
</p>

> 追加予定: アプリ一覧・ルール編集・Quick Set の静止画スクリーンショット

---

## Getting Started｜使い方

**配布パッケージは準備中です。** 現状はソースから開発ビルドで動作確認できます（下記 Development）。

公開後の想定フロー:

1. アプリをダウンロードしてインストール
2. **システム設定 › プライバシーとセキュリティ › アクセシビリティ** で ReadyType を許可
3. オンボーディングに沿って初期設定
4. Quick Set やアプリ設定でルールを登録し、普段どおり入力

---

## Tech Stack｜技術スタック

| 分類 | 技術 | 役割 |
| --- | --- | --- |
| デスクトップ | Tauri 2 | アプリ基盤 |
| UI | React 19 / TypeScript / Vite 8 | 画面 |
| スタイル | Panda CSS | トークン・スタイル |
| ルーティング | TanStack Router | 画面遷移 |
| 状態 | Jotai | UI 状態 |
| i18n | i18next | 多言語 UI |
| Native | Rust（Onion Architecture） | ドメイン・OS 連携 |
| ツール | mise / pnpm / Lefthook / Biome / Vitest | 環境・品質 |

---

## Architecture｜アーキテクチャ

```text
React / TypeScript (react-ui)
        │  invoke
        ▼
Tauri Commands (presentation)
        │
        ▼
Application / Domain (Rust)
        │
        ▼
Infrastructure（OS / 永続化）
        └── macOS
             ├── Accessibility API（入力欄の検出）
             └── Text Input Source API（切替）
```

フロントは UI、Rust はビジネスロジックと OS 依存処理を担当します。詳細は `docs/` と `AGENTS.md` を参照してください。

---

## Technical Challenges｜技術的な工夫

| 課題 | 対応 |
| --- | --- |
| 入力欄の識別 | Accessibility の複数属性を組み合わせ、ルールと照合 |
| 切替の反映漏れ | Electron 製アプリなど向けに再選択・遅延再試行 |
| Quick Set | UI がフォーカスを奪う前に対象入力欄の情報を保持 |

---

## Development｜開発環境

**前提:** macOS、[mise](https://mise.jdx.dev/)、Xcode Command Line Tools

```bash
mise install
mise run install   # react-ui の pnpm install
mise run dev       # Tauri + Vite
```

よく使うコマンド:

```bash
mise run react-ui:typecheck
mise run react-ui:fix
mise run react-ui:test
mise run rust-native:format
mise run rust-native:lint
mise run build          # 署名付き .app（macOS）
```

> Accessibility 権限が必要です。未署名ビルドでは再ビルド後に許可が外れることがあります。署名の詳細は `docs/macos-code-signing.md` を参照してください。

---

## Roadmap｜今後の展望

| 時期 | 目標 |
| --- | --- |
| 技育展 予選まで | macOS 版の一般公開、LP・ダウンロード整備 |
| 技育展 決勝まで | Windows / Linux 版の公開を目指す |
| 継続 | 切替の高速化・安定化、ルールの自動提案 |

---

## License｜ライセンス

ライセンスは公開準備中に確定予定です。条件が決まるまで、無断での再配布はご遠慮ください。

---

## Contributing｜コントリビューション

Issue / PR は歓迎します。大きな変更の前に Issue で方針を共有してもらえると助かります。コーディング規約は `AGENTS.md` / `.claude/rules/` を参照してください。
