import { css } from "styled-system/css";

export const skipLink = css({
  position: "absolute",
  left: "12px",
  top: "-60px",
  zIndex: "200",
  bg: "ink",
  color: "white",
  px: "18px",
  py: "10px",
  borderRadius: "12px",
  fontWeight: "extrabold",
  _focus: { top: "12px" },
});

export const wrap = css({
  width: "100%",
  maxWidth: "wrap",
  mx: "auto",
  px: "gutter",
});

export const section = css({
  position: "relative",
  py: "section",
  overflowX: "clip",
});

export const sectionFlush = css({
  position: "relative",
  pt: "0",
  pb: "section",
  overflowX: "clip",
});
