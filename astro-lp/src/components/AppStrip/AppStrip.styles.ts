import { css } from "styled-system/css";

export const strip = css({
  bg: "white",
  py: "22px",
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  justifyContent: "center",
  gap: "18px",
  rowGap: "10px",
  overflow: "hidden",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "line",
});

export const label = css({
  flex: "none",
  ml: "gutter",
  fontWeight: "black",
  fontSize: "0.8125rem",
  color: "muted",
  letterSpacing: "0.06em",
  whiteSpace: "nowrap",
});

export const viewport = css({
  overflow: "hidden",
  flex: "1 1 280px",
  maskImage: "linear-gradient(90deg, transparent, #000 8%, #000 92%, transparent)",
  WebkitMaskImage: "linear-gradient(90deg, transparent, #000 8%, #000 92%, transparent)",
});

export const track = css({
  display: "flex",
  gap: "12px",
  width: "max-content",
  animation: "mq2 30s linear infinite",
});

export const appChip = css({
  flex: "none",
  display: "inline-flex",
  alignItems: "center",
  gap: "8px",
  bg: "canvas",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "line",
  borderRadius: "pill",
  py: "5px",
  pr: "16px",
  pl: "6px",
  fontWeight: "extrabold",
  fontSize: "0.8438rem",
  fontStyle: "normal",
  transition: ".25s",
  _hover: { transform: "translateY(-4px)", boxShadow: "card", bg: "white" },
});
