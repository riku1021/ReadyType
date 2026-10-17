# ReadyType react-ui

Tauri 2 フロントエンド（React / Vite / Panda / TanStack Router / Jotai）。

## 開発

```bash
# リポジトリルートから
mise run install
mise run dev
```

```bash
cd tauri-desktop/react-ui
pnpm typecheck
pnpm test
pnpm fix
pnpm build
```

## 入力ソース自動切替 UI

- アクセシビリティ: `/accessibility` で権限案内・システム設定誘導
- アプリごとの設定: `/apps` でアプリ別の言語・個別ルール（`/` はここへリダイレクト）
- 一般: `/general` で自動切替 ON/OFF・ログイン起動・外観・ショートカット等
- 切替診断ログ: アプリ内 UI なし。`tmp/input-source-logs/current.jsonl`（約 10 MiB でローテート、合計約 50 MiB）と開発時のコンソール出力

手動検証: `docs/input-source-switcher-manual-test.md`  
ルールメンテ: `docs/input-source-rule-maintenance.md`

## 生成物（手編集禁止）

- `src/styled-system/`
- `src/routeTree.gen.ts`
