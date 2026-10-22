import { css } from "styled-system/css";

export const article = css({
  maxWidth: "40rem",
  py: "section",
});

export const title = css({
  fontSize: "clamp(1.75rem, 4.2vw, 2.25rem)",
  fontWeight: "black",
  lineHeight: "1.35",
  mb: "28px",
});

export const sectionHeading = css({
  fontSize: "1.125rem",
  fontWeight: "black",
  lineHeight: "1.4",
  mt: "32px",
  mb: "8px",
});

export const published = css({
  color: "muted",
  fontSize: "0.8125rem",
  mb: "20px",
});

export const paragraph = css({
  color: "muted",
  fontSize: "0.9375rem",
  mb: "12px",
});

export const clauses = css({
  m: "0",
  mb: "12px",
  pl: "1.25rem",
  listStyle: "decimal",
  color: "muted",
  fontSize: "0.9375rem",
});

export const textLink = css({
  color: "brand",
  textDecoration: "underline",
  textUnderlineOffset: "3px",
});
