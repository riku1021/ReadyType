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

- ホーム: ON/OFF・権限案内・現在状態
- ルール: `/rules` で CRUD / 並べ替え
- 診断: `/diagnostics` でログと AX スナップショット
- 設定: 既定入力ソース・ログイン起動・デバッグログ

手動検証: `docs/input-source-switcher-manual-test.md`  
ルールメンテ: `docs/input-source-rule-maintenance.md`

## 生成物（手編集禁止）

- `src/styled-system/`
- `src/routeTree.gen.ts`
