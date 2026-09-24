import { css } from "@/styled-system/css";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "md",
  maxWidth: "720px",
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
