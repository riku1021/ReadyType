import { css } from "@/styled-system/css";
import { surfaceMain } from "@/styles/objects/surface";

export const header = css(surfaceMain, {
  gridArea: "header",
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  paddingInline: "sm",
  paddingTop: "sm",
  paddingBottom: "0",
  boxSizing: "border-box",
  height: "calc(60px + {spacing.sm})",
  position: "relative",
  zIndex: "header",
  minWidth: "0",
});

export const titleRow = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minWidth: "0",
  minHeight: "1.875rem",
});

export const backButtonIcon = css({
  width: "0.875rem",
  height: "0.875rem",
  flexShrink: 0,
});

export const titleMain = css({
  display: "flex",
  alignItems: "center",
  gap: "xs",
  minWidth: "0",
  height: "1.875rem",
});

export const titleIconBox = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: 0,
  fontSize: "3xl",
  lineHeight: "0",
});

export const titleIcon = css({
  width: "1em",
  height: "1em",
  flexShrink: 0,
  color: "text.main",
});

export const title = css({
  color: "text.main",
  fontSize: "3xl",
  fontWeight: "semibold",
  lineHeight: "1.875rem",
  margin: "0",
  whiteSpace: "nowrap",
  overflow: "hidden",
  textOverflow: "ellipsis",
  minWidth: "0",
});
