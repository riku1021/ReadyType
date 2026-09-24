/**
 * Flex レイアウト
 *
 * vanilla-extract では StyleRule を spread していた。
 * Panda では実行時引数のヘルパーは抽出されないため、
 * `flex.raw` / `cva` と `css()` のマージを使う。
 */

import { cva, type RecipeVariantProps } from "@/styled-system/css";
import { flex } from "@/styled-system/patterns";

export { flex as flexLayout };

/** 水平・垂直ともに中央揃え */
export const flexCenter = flex.raw({
  direction: "row",
  wrap: "nowrap",
  justify: "center",
  align: "center",
});

/** 垂直方向のフレックス */
export const column = flex.raw({
  direction: "column",
});

/** 行方向でラップ */
export const rowWrap = flex.raw({
  direction: "row",
  wrap: "wrap",
});

/** 水平方向のみ中央揃え */
export const centerHorizontal = flex.raw({
  direction: "row",
  wrap: "nowrap",
  justify: "center",
});

/** 垂直方向のみ中央揃え */
export const centerVertical = flex.raw({
  direction: "row",
  wrap: "nowrap",
  align: "center",
});

export const spaceBetween = flex.raw({
  direction: "row",
  wrap: "nowrap",
  justify: "space-between",
});

export const spaceAround = flex.raw({
  direction: "row",
  wrap: "nowrap",
  justify: "space-around",
});

export const spaceEvenly = flex.raw({
  direction: "row",
  wrap: "nowrap",
  justify: "space-evenly",
});

/**
 * フレックス子要素。任意の basis は `css(flexChild.raw(...), { flexBasis: "200px" })`。
 */
export const flexChild = cva({
  variants: {
    grow: {
      "0": { flexGrow: "0" },
      "1": { flexGrow: "1" },
    },
    shrink: {
      "0": { flexShrink: "0" },
      "1": { flexShrink: "1" },
    },
    basis: {
      auto: { flexBasis: "auto" },
      "0": { flexBasis: "0" },
      full: { flexBasis: "100%" },
    },
  },
});

export type FlexChildVariants = RecipeVariantProps<typeof flexChild>;
