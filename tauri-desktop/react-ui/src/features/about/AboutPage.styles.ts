import { css } from "@/styled-system/css";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "0",
});

export const header = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  gap: "xs",
  paddingBlock: "md",
  paddingInline: "sm",
  textAlign: "center",
});

export const appIcon = css({
  width: "96px",
  height: "auto",
  objectFit: "contain",
});

export const appName = css({
  margin: "0",
  fontSize: "2xl",
  fontWeight: "semibold",
  lineHeight: "none",
  color: "text.main",
});

export const version = css({
  margin: "0",
  fontSize: "sm",
  color: "text.placeholder",
});

export const copyright = css({
  margin: "0",
  fontSize: "xs",
  color: "text.placeholder",
});

export const creditRow = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  width: "100%",
  paddingInline: "sm",
  // SettingItem の行高 52px に揃える。文言は約 24px なので上下 10px では足りず minHeight で揃える
  boxSizing: "border-box",
  paddingBlock: "10px",
  minHeight: "52px",
  color: "text.main",
  fontSize: "base",
});

export const creditLabel = css({
  flex: "1",
  minWidth: "0",
});

export const creditValue = css({
  flexShrink: "0",
  color: "text.placeholder",
  textAlign: "right",
});
