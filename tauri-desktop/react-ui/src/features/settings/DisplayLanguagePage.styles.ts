import { css, cva } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  minHeight: "0",
  flex: "1",
});

export const toolbar = css({
  display: "flex",
  width: "100%",
});

export const name = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const th = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineStart: "sm",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "left",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const row = css({
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  cursor: "pointer",
});

export const td = cva({
  base: {
    paddingBlock: "sm",
    paddingInline: "xs",
    paddingInlineStart: "sm",
    verticalAlign: "middle",
    minWidth: "0",
    backgroundColor: "transparent",
    borderInlineStartWidth: "3px",
    borderInlineStartStyle: "solid",
    borderInlineStartColor: "transparent",
  },
  variants: {
    selected: {
      true: {
        borderInlineStartColor: "brand.main",
      },
    },
  },
});
