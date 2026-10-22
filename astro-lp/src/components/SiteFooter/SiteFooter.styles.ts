import { css } from "styled-system/css";

export const footer = css({
  bg: "ink",
  color: "mist",
  textAlign: "left",
  pt: "72px",
  pb: "28px",
  mt: "40px",
  borderTopRadius: "44px",
});

export const grid = css({
  display: "grid",
  gridTemplateColumns: "1.6fr 1fr 1fr",
  gap: "40px",
  mb: "44px",
  _bp900: { gridTemplateColumns: "1fr" },
});

export const blurb = css({
  mt: "14px",
  fontSize: "14px",
  lineHeight: "1.9",
});

export const columnTitle = css({
  color: "white",
  fontSize: "14px",
  mb: "12px",
  lineHeight: "1.35",
  fontWeight: "black",
});

export const footerLink = css({
  display: "block",
  fontSize: "14px",
  py: "4px",
  transition: ".2s",
  _hover: { color: "white", transform: "translateX(4px)" },
});

export const legal = css({
  borderTopWidth: "1px",
  borderTopStyle: "solid",
  borderTopColor: "rgba(255, 255, 255, 0.12)",
  pt: "22px",
  fontSize: "12.5px",
});
