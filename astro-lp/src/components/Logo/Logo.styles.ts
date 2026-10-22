import { css, cva } from "styled-system/css";

export const logo = cva({
  base: {
    display: "flex",
    alignItems: "center",
    gap: "10px",
    fontFamily: "latin",
    fontWeight: "extrabold",
    fontSize: "21px",
    letterSpacing: "-0.02em",
  },
  variants: {
    tone: {
      default: { color: "ink" },
      inverse: { color: "white" },
    },
  },
  defaultVariants: { tone: "default" },
});

export const logoMark = css({
  width: "32px",
  height: "32px",
  borderRadius: "10px",
  bgGradient: "brand",
  display: "grid",
  placeItems: "center",
  color: "white",
  fontStyle: "normal",
});

export const logoAccent = cva({
  variants: {
    tone: {
      default: { color: "brand" },
      inverse: { color: "mistBlue" },
    },
  },
  defaultVariants: { tone: "default" },
});
