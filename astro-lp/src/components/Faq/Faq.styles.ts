import { css } from "styled-system/css";

export const list = css({
  maxWidth: "820px",
  mx: "auto",
  display: "grid",
  gap: "12px",
});

export const item = css({
  bg: "white",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "line",
  borderBottomWidth: "5px",
  borderRadius: "18px",
  px: "24px",
  transition: ".25s",
  _hover: { borderColor: "hoverLine" },
  "&[open]": { borderColor: "brand", boxShadow: "faq" },
});

export const summary = css({
  listStyle: "none",
  cursor: "pointer",
  py: "20px",
  fontWeight: "extrabold",
  display: "flex",
  justifyContent: "space-between",
  gap: "16px",
  "&::-webkit-details-marker": { display: "none" },
  "&::after": { content: '"＋"', color: "brand", fontWeight: "black" },
  "[open] &": {
    color: "brand",
    "&::after": { content: '"－"' },
  },
});

export const answer = css({ pb: "20px", color: "muted" });
