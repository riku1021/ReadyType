import { css, cva } from "styled-system/css";

export const logo = cva({
  base: {
    display: "flex",
    flex: "none",
    alignItems: "center",
    gap: "10px",
    fontFamily: "latin",
    fontWeight: "extrabold",
    fontSize: "1.3125rem",
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
  display: "block",
  flex: "none",
  height: "32px",
  width: "auto",
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
