---
name: exclusive-union-props
description: >-
  TypeScript で「どちらか一方の prop が必須・同時指定不可」を表現するとき、
  共通 props + 相手側を never にした排他ユニオンにする。XOR props 設計時に使う。
---

# 排他ユニオン props

双方 `optional` だと「どちらもなし」が型上許される。  
**どちらか一方必須・同時指定不可**なら共有型 + 相手 `never` のユニオンにする。

## いつ使う

- データの渡し方が排他的（`rows` | `groups`）
- 誤用を型エラーにしたい

使わない: 両方省略可、両方同時に意味がある場合。

## パターン

```ts
type SharedProps = {
  // 共通
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

## 実装

分割代入でユニオンを潰さない。`"key" in props` で絞り込んでから必須 prop に触る。

```ts
export function Component(props: Props) {
  const isGrouped = "groups" in props && props.groups !== undefined;
  if (isGrouped) {
    return props.groups.map(/* ... */);
  }
  return props.rows.map(/* ... */);
}
```

## チェックリスト

- [ ] 双方 optional の単一型にしていない
- [ ] 各枝で相手キーが `?: never`
- [ ] 公開型がユニオン
- [ ] どちらもなし・両方指定が型エラー
