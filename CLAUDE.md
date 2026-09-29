# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

詳細なコーディング規約の正本（Claude Code）は `.claude/rules/` と `.claude/skills/`。Cursor 用の並列規約は `.cursor/rules/`。Codex はルート `AGENTS.md`・`.agents/skills/`（→ `.claude/skills`）・`.codex/`。日本語で回答する。

## Project Overview

ReadyType is a **Tauri 2 desktop app** for typing practice.

**Tech Stack:**

- Frontend: React 19, Vite 8, TypeScript, TanStack Router, Panda CSS, Jotai, **i18next / react-i18next**, Biome, Vitest
- Native: Rust (Tauri), **Onion Architecture** (`domain` → `application` → `presentation` / `infrastructure`)
- Tooling: mise, pnpm, lefthook

## Development Commands

### Initial Setup

```bash
# Prerequisites: mise (Node/pnpm/Rust via toolchain)
mise install
mise run install          # pnpm install in tauri-desktop/react-ui
```

### Running Development

```bash
mise run dev              # Tauri + Vite (from react-ui)
```

### Frontend (`tauri-desktop/react-ui`)

```bash
mise run check            # pnpm typecheck
mise run lint             # Biome check
mise run fix              # Biome check --write
mise run test             # Vitest
# or:
cd tauri-desktop/react-ui && pnpm typecheck && pnpm lint && pnpm test
```

### Rust native (`tauri-desktop/rust-native`)

```bash
mise run fmt              # cargo fmt
mise run fmt:check
mise run clippy           # cargo clippy --all-targets -- -D warnings
cd tauri-desktop/rust-native && cargo test
```

### Lefthook

```bash
lefthook install          # once per clone
# pre-commit: Biome fix + typecheck + rustfmt + clippy
```

## Architecture

### Frontend (`tauri-desktop/react-ui/`)

Bulletproof React–style features + Panda CSS (aligned with CariBan shell):

```
src/
├── features/       # home, settings, navigation, …
├── components/     # Button, Dialog, FormModal, Lottie, ThemeToggle
├── layouts/        # RootLayout, Drawer, Header
├── routes/         # TanStack Router (thin)
├── locales/        # i18next（ja / en、preference system|ja|en）
├── styles/         # Panda tokens / globalCss
├── atoms/ hooks/
└── styled-system/  # codegen only — do not edit
```

- Client UI state: Jotai
- Native bridge: `@tauri-apps/api` `invoke`
- UI copy: **i18next**（`useTranslation` / `t()`）。ハードコード禁止。詳細は `.claude/rules/react-frontend.md` §14 と `docs/rules/ja-JP/react-frontend-i18n.md`
- Do **not** add React Query / axios without approval

### Native (`tauri-desktop/rust-native/`) — Onion Architecture

```
src/
├── domain/           # entities, VO, DomainError (innermost)
├── application/      # use_cases + outbound ports (traits)
├── presentation/     # Tauri commands (IPC adapters)
├── infrastructure/   # OS / persistence implementations
├── lib.rs            # composition root
└── main.rs
```

**Dependency rule:** outer → inner only. `domain` must not import Tauri/OS/DB.

## Code Style

### TypeScript / React

- Biome for lint/format (`pnpm fix`)
- No `any` / `unknown`; prefer named exports; avoid `class` unless necessary
- Path alias `@/` → `src/`
- Component directories: PascalCase
- All user-facing text via i18next (`t()`). Keep `ja` / `en` JSON in sync. Locale preference may be `system` (resolve to `ja`|`en` for UI/tray)
- UI/UX changes require approval

### Rust

- `cargo fmt` + Clippy (`-D warnings`)
- Thin Tauri commands; business logic in use cases
- Map `DomainError` to IPC-friendly errors in presentation

## Generated / Do Not Hand-Edit

- `tauri-desktop/react-ui/src/styled-system/`
- `tauri-desktop/react-ui/src/routeTree.gen.ts`

## Agents, Rules & Skills

- Subagents（Claude）: `.claude/agents/`（coder / quality-checker）
- Rules（詳細・正本）: `.claude/rules/`
  - `agent-behavior`（always）、`rust-backend`、`rust-testing`
  - `react-frontend`（**i18n 含む**）、`panda-css`、`typescript-general`、`typescript-testing`
- 人間向け i18n ガイド: `docs/rules/ja-JP/react-frontend-i18n.md`
- 入力ソース / LanguageTag モデル: `docs/rules/ja-JP/input-source-model.md`
- Skills（Claude）: `.claude/skills/`
- Codex:
  - 指示: ルート `AGENTS.md`（＋ `react-ui` / `rust-native` 配下のネスト）
  - Skills: `.agents/skills/`（各エントリは `.claude/skills/` への symlink）
  - Config / コマンド承認: `.codex/config.toml`、`.codex/rules/*.rules`
- Cursor 並列: `.cursor/rules/` / `.cursor/skills/`
