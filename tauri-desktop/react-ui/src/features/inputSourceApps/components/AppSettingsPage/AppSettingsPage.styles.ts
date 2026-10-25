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
  gap: "xs",
  width: "100%",
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

export const tdApp = cva({
  base: {
    paddingBlock: "sm",
    paddingInline: "xs",
    paddingInlineStart: "sm",
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

export const sortControl = css({
  position: "relative",
  display: "inline-flex",
});

export const sortVisual = css({
  display: "inline-flex",
  pointerEvents: "none",
});

/** 見た目は neutral ボタン。開く一覧は LanguageSelect（入力言語と同じネイティブの select）。 */
export const sortSelectHit = css({
  position: "absolute",
  inset: "0",
  width: "100%",
  height: "100%",
  maxWidth: "100%",
  minWidth: "0",
  margin: "0",
  opacity: "0",
  cursor: "pointer",
  boxSizing: "border-box",
  outline: "none",
  _focus: {
    outline: "none",
  },
  _focusVisible: {
    outline: "none",
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
