/**
 * レイアウトヘルパーの使用例
 * 上書きは cx ではなく css() でマージする。
 */

import { css } from "@/styled-system/css";

import {
  centerVertical,
  column,
  flexCenter,
  flexChild,
  flexLayout,
  rowWrap,
  spaceBetween,
} from "../layouts/flex";
import { equalColumns, gridCenter, gridChild, gridLayout, responsive } from "../layouts/grid";

export const centeredViewport = css(flexCenter, {
  height: "100vh",
});

export const headerBar = css(spaceBetween, centerVertical, {
  paddingInline: "sm",
});

export const sidebar = css(column, {
  gap: "sm",
});

export const cardRow = css(rowWrap, {
  gap: "md",
});

export const flexItem = css(flexChild.raw({ grow: "1", shrink: "0", basis: "0" }), {
  flexBasis: "200px",
});

export const customFlex = css(
  flexLayout.raw({
    direction: "row",
    wrap: "wrap",
    justify: "space-between",
    align: "center",
  }),
  { gap: "md", padding: "lg" }
);

export const cardGrid = css(responsive, {
  padding: "md",
});

export const threeColumns = css(equalColumns.raw({ count: 3, gap: "md" }));

export const centeredBlock = css(gridCenter, {
  height: "100vh",
});

export const pageGrid = css(
  gridLayout.raw({
    columns: 2,
    gap: "md",
  })
);

export const wideItem = css(
  gridChild.raw({
    colStart: 1,
    colEnd: 3,
  })
);

export const namedPage = css({
  display: "grid",
  gridTemplateAreas: `"header header" "drawer main"`,
  gridTemplateColumns: "250px 1fr",
  gridTemplateRows: "60px 1fr",
});

export const headerArea = css({
  gridArea: "header",
});
