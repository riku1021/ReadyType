import { css } from "@/styled-system/css";

export const form = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  minWidth: "320px",
  maxWidth: "480px",
});

export const field = css({
  display: "flex",
  flexDirection: "column",
  gap: "4px",
});

export const label = css({
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const input = css({
  fontSize: "sm",
  paddingBlock: "xs",
  paddingInline: "sm",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "bg.main",
  color: "text.main",
});

export const error = css({
  margin: "0",
  fontSize: "sm",
  color: "status.error.main",
});

export const conditionBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  padding: "sm",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
});

export const footer = css({
  display: "flex",
  justifyContent: "flex-end",
  gap: "xs",
});
