import { css } from "styled-system/css";

export const grid = css({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 240px), 1fr))",
  gap: "clamp(16px, 2.4vw, 22px)",
});

export const card = css({
  minHeight: "200px",
  borderWidth: "2px",
  borderStyle: "dashed",
  borderColor: "reviewLine",
  borderRadius: "card",
  backgroundImage: "linear-gradient(180deg, #fff, #F4F7FF)",
  display: "grid",
  placeItems: "center",
  color: "reviewText",
  fontWeight: "extrabold",
  fontSize: "0.875rem",
  textAlign: "center",
  p: "24px",
  gap: "6px",
  transition: ".25s",
  whiteSpace: "pre-line",
  _hover: { transform: "translateY(-6px)", borderColor: "brand" },
});

export const stars = css({
  display: "flex",
  gap: "0.2em",
  color: "brand",
  fontSize: "1.125rem",
});
