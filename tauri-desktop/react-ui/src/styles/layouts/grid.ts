/**
 * Grid レイアウト
 *
 * vanilla-extract では StyleRule を spread していた。
 * Panda では実行時引数のヘルパーは抽出されないため、
 * `grid.raw` / `gridItem.raw` / `cva` と `css()` のマージを使う。
 *
 * 名前付きエリアは `css({ gridTemplateAreas: `"header header" "drawer main"` })`。
 * 子の area は `css({ gridArea: "header" })`。
 */

import { css, cva, type RecipeVariantProps } from "@/styled-system/css";
import { grid, gridItem } from "@/styled-system/patterns";

export { grid as gridLayout, gridItem as gridChild };

/** 単一アイテムの中央配置 */
export const gridCenter = css.raw({
  display: "grid",
  placeItems: "center",
});

/**
 * 自動レスポンシブグリッド（Panda は auto-fit）。
 * 別の最小幅は呼び出し側で `gridLayout.raw({ minChildWidth: "250px", gap: "md" })`。
 */
export const responsive = grid.raw({
  minChildWidth: "200px",
  gap: "sm",
});

/** 均等カラム。任意の列数は `gridLayout.raw({ columns: 3, gap: "md" })` でも可 */
export const equalColumns = cva({
  base: {
    display: "grid",
  },
  variants: {
    count: {
      2: { gridTemplateColumns: "repeat(2, minmax(0, 1fr))" },
      3: { gridTemplateColumns: "repeat(3, minmax(0, 1fr))" },
      4: { gridTemplateColumns: "repeat(4, minmax(0, 1fr))" },
      5: { gridTemplateColumns: "repeat(5, minmax(0, 1fr))" },
      6: { gridTemplateColumns: "repeat(6, minmax(0, 1fr))" },
    },
    gap: {
      xs: { gap: "xs" },
      sm: { gap: "sm" },
      md: { gap: "md" },
      lg: { gap: "lg" },
      xl: { gap: "xl" },
    },
  },
  defaultVariants: {
    count: 2,
    gap: "sm",
  },
});

/** 均等行 */
export const equalRows = cva({
  base: {
    display: "grid",
  },
  variants: {
    count: {
      2: { gridTemplateRows: "repeat(2, minmax(0, 1fr))" },
      3: { gridTemplateRows: "repeat(3, minmax(0, 1fr))" },
      4: { gridTemplateRows: "repeat(4, minmax(0, 1fr))" },
    },
    gap: {
      xs: { gap: "xs" },
      sm: { gap: "sm" },
      md: { gap: "md" },
      lg: { gap: "lg" },
      xl: { gap: "xl" },
    },
  },
  defaultVariants: {
    count: 2,
    gap: "sm",
  },
});

export type EqualColumnsVariants = RecipeVariantProps<typeof equalColumns>;
export type EqualRowsVariants = RecipeVariantProps<typeof equalRows>;
