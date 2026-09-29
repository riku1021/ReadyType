---
description: Vitest による React UI のテスト規約（配置・AAA・mockIPC・カバレッジ）
globs:
  - "tauri-desktop/react-ui/**/*.{ts,tsx}"
alwaysApply: false
---

# TypeScript Testing Rules（Vitest）

対象: `tauri-desktop/react-ui/`。Rust 側は `rust-testing` を参照。

## Testing Stack

| 項目 | 内容 |
|------|------|
| Runner | Vitest |
| 環境 | プロジェクトの `vitest.config` / Vite 設定に従う |
| Tauri | `@tauri-apps/api` の `mockIPC` 等（導入済みならそれを使う） |
| Assertion | Vitest の `expect` |

React Testing Library の追加・E2E ツール追加は **承認必須**。

## Commands

```bash
cd tauri-desktop/react-ui
pnpm test
pnpm test -- --watch
pnpm test -- --coverage   # スクリプトがある場合
```

## 方針

1. **純粋関数・変換・小さなフック**を優先してユニットテストする
2. Tauri 依存は `mockIPC`（または同等）で隔離する
3. 見た目のスナップショット量産をしない
4. Rust の domain テストと重複するビジネスルールをフロントで二重管理しない

## File Organization

- 配置: `tauri-desktop/react-ui/tests/` に `src/` と同じ相対パスをミラーする（例: `src/features/foo/bar.ts` → `tests/features/foo/bar.test.ts`）
- `src` 内への `*.test.ts` コロケーションはしない
- テストファイル名: `*.test.ts` / `*.test.tsx`
- 被テストモジュールは `@/` エイリアスで import する
- E2E（Playwright）は `e2e/`（本ルールの Vitest 配置とは別）
- テスト専用ユーティリティは `tests/` 配下、または既存の helper 配置に合わせる

## Arrange / Act / Assert

```ts
import { describe, expect, it } from "vitest";
import { formatLabel } from "@/features/example/formatLabel";

describe("formatLabel", () => {
  it("trims and returns label", () => {
    // Arrange
    const input = "  ReadyType  ";

    // Act
    const result = formatLabel(input);

    // Assert
    expect(result).toBe("ReadyType");
  });
});
```

## Table-Driven Style

```ts
describe("isValidName", () => {
  it.each([
    { input: "Alice", expected: true },
    { input: "", expected: false },
    { input: "   ", expected: false },
  ])("$input → $expected", ({ input, expected }) => {
    expect(isValidName(input)).toBe(expected);
  });
});
```

## Tauri invoke のモック

```ts
import { mockIPC } from "@tauri-apps/api/mocks";
import { beforeEach, describe, expect, it } from "vitest";
import { greet } from "@/features/example/greet";

describe("greet", () => {
  beforeEach(() => {
    mockIPC((cmd, args) => {
      if (cmd === "greet") {
        return `Hello, ${(args as { name: string }).name}!`;
      }
      throw new Error(`unexpected command: ${cmd}`);
    });
  });

  it("calls greet command", async () => {
    await expect(greet("World")).resolves.toContain("World");
  });
});
```

- 未想定コマンドは fail させる
- 実際の Rust バイナリに依存しない

## Component Tests（入れる場合）

- ユーザー操作に近いレベルで最小限
- 実装詳細（内部 state）より表示とコールバックを検証
- Panda の class 名の完全一致に依存しない（壊れやすい）

## What to Test / Not Test

### Test

- util / 変換ロジック
- atom の更新ルール（複雑な場合）
- invoke ラッパの引数・エラー伝播
- 排他的 props の型は型テストまたはコンパイルで担保（実行時テストは最小）

### Avoid

- `styled-system` 生成物
- `routeTree.gen.ts`
- 三流の「レンダーできた」だけのテストの量産
- サードパーティ内部の再テスト

## Async

```ts
it("handles invoke failure", async () => {
  mockIPC(() => {
    throw new Error("boom");
  });
  await expect(greet("x")).rejects.toThrow();
});
```

- `waitFor` が必要なら公式推奨 API に従う（RTL 導入後）
- タイマーは `vi.useFakeTimers` を明示的に使う

## Coverage（目安）

- utils / 変換: 高
- invoke ラッパ: 代表的成功・失敗
- ページコンポーネント全体: 低〜中（重要フローのみ）

カバレッジ数字のための無意味テストは作らない。

## Anti-Patterns

- テスト間で共有ミュータブル状態をリセットしない
- 実ネットワーク・実ファイルへの依存
- スナップショットの巨大化
- `any` で型エラーを回避したテストコード

## Checklist（新機能）

1. 純粋ロジックに `tests/` 配下へ `*.test.ts` を追加（`src` ミラー）
2. `invoke` を使うなら mockIPC ケースを追加
3. `pnpm test` と `pnpm typecheck` を通す
4. Rust 側で担保すべきルールは `cargo test` に寄せる
