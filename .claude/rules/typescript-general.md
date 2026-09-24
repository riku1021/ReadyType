---
description: TypeScript / Biome の一般規約（型・品質・セキュリティ・ワークフロー）
globs:
  - "tauri-desktop/react-ui/**/*.{ts,tsx}"
alwaysApply: false
---

# TypeScript General Rules

対象: `tauri-desktop/react-ui/`（および同梱の TS 設定）。

## TypeScript Version & Config

- TypeScript はパッケージ / mise で固定されたバージョンを使う
- `strict` を前提とする
- path alias: `@/*` → `src/*`
- `baseUrl` に依存した古い書き方を増やさない（paths で足りる）

## Type Safety

### 禁止

- `any`
- `unknown`（どうしても境界で必要な場合は、すぐ狭い型へ絞り込む設計にし、安易に増やさない）
- 不合理な `as` キャストの連発
- `@ts-ignore` / `@ts-expect-error` の恒久利用（一時利用は理由コメント必須）

### 推奨

- `interface` / `type` で Props とドメイン形を明示
- `satisfies` でオブジェクトリテラルの検証
- 判別共用体（discriminated union）
- 相互排他 props は `exclusive-union-props` スキル

```ts
type Success = { ok: true; data: string };
type Failure = { ok: false; error: string };
type Result = Success | Failure;

function message(result: Result): string {
  if (result.ok) {
    return result.data;
  }
  return result.error;
}
```

### 関数

- 引数・戻り値に型を付ける（推論で十分なローカルなら省略可だが、公開 API は明示）
- ジェネリクスは実際に再利用・拘束があるときだけ

## Code Quality（Biome）

Biome が ESLint / Prettier の代替。設定: `tauri-desktop/react-ui/biome.json`。

### 実設定の要点

| 項目 | 値 |
|------|-----|
| indent | space / width 2 |
| lineWidth | 100 |
| quotes | double（JS/JSX） |
| semicolons | always |
| trailingCommas | es5 |
| `noExplicitAny` | error |
| `useExhaustiveDependencies` | error |
| `noDeprecatedImports` | error |

### 除外

- `src/styled-system/**`
- `src/routeTree.gen.ts`
- `dist` / lockfile / markdown

### コマンド

```bash
cd tauri-desktop/react-ui
pnpm fix          # lint + format 相当
pnpm check        # CI 向けチェック（スクリプトに従う）
pnpm typecheck
```

## Import / Module

- `@/` alias を使う
- feature / component は `index.ts` の公開 API 経由
- 生成物（`styled-system`、`routeTree.gen.ts`）を手編集しない
- 循環依存を作らない
- 使っていない import は残さない（Biome に任せる）

## Naming

| 対象 | 規則 |
|------|------|
| コンポーネント dir | PascalCase（`Button/`） |
| コンポーネントファイル | PascalCase（`Button.tsx`） |
| styles | `ComponentName.styles.ts` |
| hooks | `useXxx.ts` |
| atoms | `xxxAtom.ts` など既存に合わせる |
| 通常の util | camelCase |

新規コンポーネント dir のケーシング手順: `component-path-casing` スキル。

## Error Handling

- 非同期は `try/catch` または Result 型で明示
- UI 境界ではユーザー向けメッセージに変換
- 空 catch 禁止
- Tauri `invoke` の失敗を握りつぶさない

## Security

- 秘密情報をソース・ログに出さない
- ユーザー入力をそのまま `dangerouslySetInnerHTML` しない
- 永続・OS 操作は Rust 側でも検証する前提で UI 検証を書く

## Performance（過度な最適化はしない）

- 大きなリストは必要になってから仮想化を検討
- 不用意な全アプリ再レンダーを避ける（状態のスコープを狭く）
- 依頼されていない memo の大量追加はしない（React Compiler 方針があればそれに従う）

## Documentation

- 複雑な公開関数のみ簡潔にコメント
- 自明なコメントを増やさない
- README / CLAUDE.md の手順と矛盾させない

## Development Workflow

```bash
cd tauri-desktop/react-ui
pnpm install
pnpm dev
pnpm build
pnpm fix
pnpm typecheck
pnpm test
pnpm exec panda codegen
```

mise / pnpm のバージョンはリポジトリの固定に従う。

## Vite

- 開発サーバーは Tauri 連携（port 1420）
- プラグイン: React、TanStack Router、Panda PostCSS 等（既存設定を維持）
- 新規プラグイン追加は承認必須

## Best Practices

- コンポーネントを純粋に保つ（可能な範囲）
- 単一責任
- 既存パターンに合わせる（好みで書き直さない）
- 依頼範囲外のリファクタをしない（`agent-behavior`）

## Anti-Patterns

- `any` / 安易な `unknown`
- CSS Modules / SCSS の追加
- 生成ファイルの手編集
- 承認なしの大規模依存追加
- テストや型を弱めて「とりあえず通す」
