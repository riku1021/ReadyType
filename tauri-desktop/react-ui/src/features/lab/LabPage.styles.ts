import { css } from "@/styled-system/css";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "md",
});

export const heading = css({
  margin: "0",
  fontSize: "3xl",
  fontWeight: "semibold",
  color: "text.main",
});

export const lead = css({
  margin: "0",
  fontSize: "base",
  lineHeight: "relaxed",
  color: "text.sub",
});

export const sectionTitle = css({
  margin: "0",
  fontSize: "xl",
  fontWeight: "semibold",
  color: "text.main",
});

export const buttonGroup = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "sm",
});

export const result = css({
  color: "text.sub",
  fontSize: "sm",
});

export const formBody = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  color: "text.sub",
  fontSize: "base",
});

export const fieldList = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  margin: "0",
  paddingInlineStart: "md",
  listStyleType: "disc",
});

export const fieldRow = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
});

export const fieldLabel = css({
  fontSize: "sm",
  fontWeight: "medium",
  color: "text.main",
});

export const fieldHint = css({
  fontSize: "xs",
  color: "text.secondary",
});
