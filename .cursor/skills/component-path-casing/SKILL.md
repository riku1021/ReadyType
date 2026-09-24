---
name: component-path-casing
description: >-
  macOS の大文字小文字非区別 FS で、コンポーネントディレクトリの PascalCase を保ち、
  foo/Foo のようなケーシング事故を防ぐ。コンポーネント新規作成・リネーム・移動、
  import パス変更、ディレクトリ整理のときに使う。
---

# コンポーネントパスのケーシング

CariBan のコンポーネントディレクトリは **PascalCase** が正（例: `src/components/SegmentedControl/`、`features/<name>/components/FooCard/`）。

以下の `<Name>` は意図する PascalCase 名、`<name>` はその lowercase 版を表す。

## なぜ事故るのか

macOS のデフォルト（APFS case-insensitive）では `<name>` と `<Name>` は同じディレクトリ扱いになる。

- 先に `<name>/` を作ると、あとから `<Name>/` と書いても **実体の名前は `<name>` のまま**
- import が `@/components/<Name>` でも動くが、Finder / git / IDE では `<name>` に見える
- `git mv <name> <Name>` は **一発では失敗する**（同じパスとみなされる）

## 必須ルール

1. **新規コンポーネントディレクトリは最初から PascalCase**  
   `<Name>/` のように。lowercase `<name>/` を作らない。
2. **ディレクトリ名と import を一致させる**  
   実体が `<Name>/` なら `@/components/<Name>`（または相対パスでも同じ表記）。
3. **作成・リネーム後は実ケーシングを検証する**（下記）

## 新規作成チェックリスト

```text
- [ ] ディレクトリ名は PascalCase（既存の SegmentedControl 等に合わせる）
- [ ] 配下ファイルも慣例どおり（ComponentName.tsx, ComponentName.styles.ts, index.ts）
- [ ] import パスの大文字小文字がディレクトリと一致
- [ ] 検証コマンドで listdir の表記が意図どおり
```

## 検証（必須）

`ls` だけだと IDE 表示と食い違うことがある。Python で **実際のエントリ名** を確認する:

```bash
# PARENT = 親ディレクトリ（例: tauri-desktop/react-ui/src/components）
# NAME_LOWER = 確認したい名前の小文字（例: segmentedcontrol）
python3 -c "import os; print([x for x in os.listdir('PARENT') if x.lower()=='NAME_LOWER'])"
```

期待: `['<Name>']`（意図した PascalCase そのもの）

## リネーム手順（case-only）

大文字小文字だけ変えたいとき（`<name>` → `<Name>`）:

```bash
# PARENT = 親ディレクトリ
# 1) 中間名へ（必須。直接 <name>→<Name> は失敗する）
mv PARENT/<name> PARENT/<name>-tmp
# 2) 最終名へ
mv PARENT/<name>-tmp PARENT/<Name>
# 3) 検証（NAME_LOWER は <Name> の lower）
python3 -c "import os; print([x for x in os.listdir('PARENT') if x.lower()=='NAME_LOWER'])"
```

git 管理下なら同様に **二段階** `git mv`:

```bash
git mv PARENT/<name> PARENT/<name>-tmp
git mv PARENT/<name>-tmp PARENT/<Name>
```

未追跡ディレクトリは通常の `mv` 二段階でよい。

## やってはいけないこと

- case-insensitive 環境で `<name>/` を作ってから「`<Name>` に直したつもり」で終わる
- import だけ `@/.../<Name>` にしてディレクトリ実体を放置する
- 一発の `mv <name> <Name>` / `git mv <name> <Name>` で直ったとみなす

## 関連

- コンポーネント配置の責務: `.cursor/rules/frontend/structure.mdc`
- 実装時の最小 diff: `.cursor/skills/agent-behavior/SKILL.md`
