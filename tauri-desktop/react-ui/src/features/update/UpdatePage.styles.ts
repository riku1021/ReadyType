import { css } from "@/styled-system/css";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  maxWidth: "640px",
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
