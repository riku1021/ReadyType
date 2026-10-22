import { css } from "styled-system/css";

export const footer = css({
  bg: "ink",
  color: "mist",
  textAlign: "left",
  pt: "72px",
  pb: "28px",
  mt: "40px",
  borderTopRadius: "44px",
  overflowX: "clip",
});

export const grid = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "clamp(28px, 5vw, 40px)",
  mb: "44px",
  "& > :first-child": { flex: "1.6 1 260px" },
  "& > :not(:first-child)": { flex: "1 1 140px" },
});

export const blurb = css({
  mt: "14px",
  fontSize: "0.875rem",
  lineHeight: "1.9",
});

export const columnTitle = css({
  color: "white",
  fontSize: "0.875rem",
  mb: "12px",
  lineHeight: "1.35",
  fontWeight: "black",
});

export const footerLink = css({
  display: "block",
  fontSize: "0.875rem",
  py: "4px",
  transition: ".2s",
  _hover: { color: "white", transform: "translateX(4px)" },
});

export const legal = css({
  borderTopWidth: "1px",
  borderTopStyle: "solid",
  borderTopColor: "rgba(255, 255, 255, 0.12)",
  pt: "22px",
  fontSize: "0.7812rem",
});
