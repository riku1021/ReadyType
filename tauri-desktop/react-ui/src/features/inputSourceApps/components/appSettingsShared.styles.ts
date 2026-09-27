import { css } from "@/styled-system/css";

export const appIcon = css({
  width: "36px",
  height: "36px",
  borderRadius: "9px",
  flexShrink: "0",
  objectFit: "cover",
  display: "block",
});

export const appIconLarge = css({
  width: "56px",
  height: "56px",
  borderRadius: "13px",
  flexShrink: "0",
  objectFit: "cover",
  display: "block",
});

export const appIconFallback = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "36px",
  height: "36px",
  borderRadius: "9px",
  flexShrink: "0",
  backgroundColor: "background.sub",
  color: "text.secondary",
  fontWeight: "semibold",
  fontSize: "sm",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
});

export const appIconFallbackLarge = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "56px",
  height: "56px",
  borderRadius: "13px",
  flexShrink: "0",
  backgroundColor: "background.sub",
  color: "text.secondary",
  fontWeight: "semibold",
  fontSize: "xl",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
});

export const select = css({
  paddingInline: "sm",
  paddingBlock: "xs",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.sub",
  color: "text.main",
  fontSize: "sm",
  cursor: "pointer",
});

export const selectWide = css({
  width: "100%",
  maxWidth: "360px",
});

export const ruleIconBox = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "34px",
  height: "34px",
  borderRadius: "9px",
  flexShrink: "0",
  backgroundColor: "background.sub",
  color: "text.main",
  fontSize: "lg",
});
