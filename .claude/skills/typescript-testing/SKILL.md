---
name: typescript-testing
description: >-
  ReadyType Vitest（配置・AAA・mockIPC・何をテストするか）。
  react-ui のテスト追加・修正時に使う。
---

# TypeScript Testing Skill

正本ルール: `.claude/rules/typescript-testing.md`。Rust は `rust-testing`。

## 方針

1. 純粋関数・変換・薄い invoke ラッパを優先
2. Tauri は `mockIPC` で隔離
3. スナップショット量産しない
4. ドメイン不変条件の本丸は Rust テスト

## コマンド

```bash
cd tauri-desktop/react-ui
pnpm test
```

## テンプレ

```ts
import { describe, expect, it } from "vitest";

describe("formatLabel", () => {
  it("trims", () => {
    expect(formatLabel("  x  ")).toBe("x");
  });
});
```

## mockIPC

```ts
import { mockIPC } from "@tauri-apps/api/mocks";
import { beforeEach, expect, it } from "vitest";

beforeEach(() => {
  mockIPC((cmd, args) => {
    if (cmd === "greet") {
      return `Hello, ${(args as { name: string }).name}!`;
    }
    throw new Error(`unexpected: ${cmd}`);
  });
});
```

## Done

- [ ] 成功・失敗の代表ケース
- [ ] `pnpm test` / `pnpm typecheck`
- [ ] 生成物や三流レンダーテストを増やしていない
