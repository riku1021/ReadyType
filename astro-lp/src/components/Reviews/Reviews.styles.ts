import { css } from "styled-system/css";

export const grid = css({
  display: "grid",
  gridTemplateColumns: "repeat(3, 1fr)",
  gap: "22px",
  _bp900: { gridTemplateColumns: "1fr" },
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
  fontSize: "14px",
  textAlign: "center",
  p: "24px",
  gap: "6px",
  transition: ".25s",
  whiteSpace: "pre-line",
  _hover: { transform: "translateY(-6px)", borderColor: "brand" },
  "&::before": {
    content: '"★★★★★"',
    display: "block",
    letterSpacing: "0.2em",
    textGradient: "brand",
    fontSize: "18px",
  },
});
