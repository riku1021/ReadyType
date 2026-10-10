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
- 診断: `/diagnostics` でログと AX スナップショット
- 一般: `/general` で自動切替 ON/OFF・ログイン起動・外観・ショートカット等

手動検証: `docs/input-source-switcher-manual-test.md`  
ルールメンテ: `docs/input-source-rule-maintenance.md`

## 生成物（手編集禁止）

- `src/styled-system/`
- `src/routeTree.gen.ts`
