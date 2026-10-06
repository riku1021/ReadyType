import { css } from "@/styled-system/css";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "0",
  maxWidth: "640px",
  paddingBottom: "md",
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
  height: "96px",
  borderRadius: "22px",
  objectFit: "cover",
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
  paddingBlock: "sm",
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
