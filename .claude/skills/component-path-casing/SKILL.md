---
name: component-path-casing
description: >-
  macOS の大文字小文字非区別 FS で、コンポーネントディレクトリの PascalCase を保ち、
  foo/Foo のようなケーシング事故を防ぐ。コンポーネント新規作成・リネーム・移動時に使う。
---

# コンポーネントパスのケーシング

ReadyType（`tauri-desktop/react-ui`）のコンポーネントディレクトリは **PascalCase** が正  
（例: `src/components/Button/`、`features/<name>/components/HomePage/`）。

`<Name>` = 意図する PascalCase、`<name>` = lowercase。

## なぜ事故るか

APFS case-insensitive では `<name>` と `<Name>` は同じディレクトリ扱い。

- 先に `<name>/` を作ると実体名は `<name>` のまま
- import が `@/components/<Name>` でも動くが git / Finder では `<name>`
- `git mv <name> <Name>` は一発失敗する

## 必須

1. 新規は最初から PascalCase
2. ディレクトリ名と import 表記を一致
3. 作成・リネーム後に実ケーシングを検証

## 検証

```bash
python3 -c "import os; print([x for x in os.listdir('PARENT') if x.lower()=='NAME_LOWER'])"
```

期待: `['<Name>']`

## case-only リネーム（二段階必須）

```bash
mv PARENT/<name> PARENT/<name>-tmp
mv PARENT/<name>-tmp PARENT/<Name>
```

git:

```bash
git mv PARENT/<name> PARENT/<name>-tmp
git mv PARENT/<name>-tmp PARENT/<Name>
```

## やってはいけない

- lowercase で作ってから「直したつもり」で終わる
- import だけ PascalCase にして実体放置
- 一発 `mv` / `git mv` で直ったとみなす

## 関連

- `.claude/rules/react-frontend.md`
- `.claude/skills/agent-behavior/SKILL.md`
