import { css } from "@/styled-system/css";

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

export const row = css({
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "border.primary",
  cursor: "pointer",
});

export const td = css({
  paddingBlock: "sm",
  paddingInline: "xs",
  paddingInlineStart: "sm",
  verticalAlign: "middle",
  minWidth: "0",
  backgroundColor: "transparent",
});
