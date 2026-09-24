---
name: exclusive-union-props
description: >-
  TypeScript で「どちらか一方の prop が必須・同時指定不可」を表現するとき、
  共通 props + 相手側を never にした排他ユニオンにする。
  XOR props、either-or props、rows|groups、variant A|B の API 設計・型定義時に使う。
---

# 排他ユニオン props（どちらか一方必須）

双方を `optional` にすると「どちらもなし」が型上許される。
**どちらか一方が必須で、同時指定は不可**なら、共有型 + 相手を `never` にしたユニオンにする。

## いつ使うか

- データの渡し方が排他的（例: `rows` か `groups`）
- UI モードが排他的（例: `value` か `defaultValue` を厳密に分けたいとき以外の、独自 XOR）
- 「優先順位で片方を無視」より、呼び出し側で誤用を型エラーにしたいとき

使わない:

- 本当に両方省略可
- 両方同時に意味がある（合成可能な独立オプション）

## パターン

```ts
type SharedProps = {
  // 共通の必須・任意
};

type VariantAProps = SharedProps & {
  rows: Row[];
  groups?: never;
};

type VariantBProps = SharedProps & {
  groups: Group[];
  rows?: never;
};

/** A か B のどちらか一方が必須（同時指定不可） */
export type Props = VariantAProps | VariantBProps;
```

要点:

1. 共通は `SharedProps` に切り出す
2. 各バリアントで「自分の必須 prop」を宣言し、**相手のキーは `?: never`**
3. 公開型はユニオン。コメントで「どちらか一方必須」を明示する

## コンポーネント実装

分割代入で `rows` / `groups` を両方取るとユニオンが潰れることがある。
**discriminant で絞り込んでから** 各枝の必須 prop に触る。

```ts
export function Component(props: Props) {
  const { /* SharedProps のみ分割代入 */ } = props;

  const isGrouped = "groups" in props && props.groups !== undefined;

  if (isGrouped) {
    // ここでは props.groups が必須扱い
    return props.groups.map(/* ... */);
  }

  // ここでは props.rows が必須扱い
  return props.rows.map(/* ... */);
}
```

## チェックリスト

```text
- [ ] 双方 optional の単一オブジェクト型にしていない
- [ ] 各枝で相手キーが `?: never`
- [ ] 公開型がユニオンで export されている
- [ ] 実装側は `"key" in props` 等で絞り込んでから必須 prop を参照
- [ ] どちらもなし・両方指定が型エラーになることを確認
```

## 参照実装

ReadyType に該当コンポーネントが無い場合は、本スキルのテンプレをそのまま使う。
（CariBan では `DataTable` の `rows` | `groups` が同パターン。）
