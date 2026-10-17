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
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  flexWrap: "wrap",
});

export const intro = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
});

export const toolbarActions = css({
  display: "flex",
  alignItems: "center",
  gap: "xs",
  flexWrap: "wrap",
});

export const appCell = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minWidth: "0",
});

export const appName = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
  minWidth: "0",
});

export const ruleCount = css({
  fontSize: "sm",
  color: "text.main",
});

export const th = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "left",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const thApp = css(surfaceSub, {
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

export const thEnabled = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "right",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  whiteSpace: "nowrap",
});

export const thChevron = css(surfaceSub, {
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineEnd: "sm",
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.sub",
  textAlign: "right",
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
    verticalAlign: "middle",
    minWidth: "0",
    backgroundColor: "transparent",
  },
  variants: {
    selected: {
      true: {
        backgroundColor: "background.hover",
      },
    },
  },
});

export const tdApp = cva({
  base: {
    paddingBlock: "sm",
    paddingInline: "xs",
    paddingInlineStart: "sm",
    verticalAlign: "middle",
    minWidth: "0",
    backgroundColor: "transparent",
    borderLeftWidth: "3px",
    borderLeftStyle: "solid",
    borderLeftColor: "transparent",
  },
  variants: {
    selected: {
      true: {
        backgroundColor: "background.hover",
        borderLeftColor: "brand.main",
      },
    },
  },
});

export const tdEnabled = cva({
  base: {
    paddingBlock: "sm",
    paddingInline: "xs",
    verticalAlign: "middle",
    minWidth: "0",
    backgroundColor: "transparent",
    textAlign: "right",
  },
  variants: {
    selected: {
      true: {
        backgroundColor: "background.hover",
      },
    },
  },
});

export const tdChevron = cva({
  base: {
    paddingBlock: "sm",
    paddingInline: "xs",
    paddingInlineEnd: "sm",
    verticalAlign: "middle",
    minWidth: "0",
    backgroundColor: "transparent",
    textAlign: "right",
  },
  variants: {
    selected: {
      true: {
        backgroundColor: "background.hover",
      },
    },
  },
});

export const chevron = css({
  display: "inline-flex",
  width: "24px",
  height: "24px",
  fontSize: "24px",
  color: "icon.secondary",
  "& > svg": {
    width: "100%",
    height: "100%",
  },
});
