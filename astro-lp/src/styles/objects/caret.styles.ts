import { cva } from "styled-system/css";

export const caret = cva({
  base: {
    display: "inline-block",
    width: "3px",
    height: "1.2em",
    ml: "3px",
    animation: "blink 1s steps(1) infinite",
  },
  variants: {
    tone: {
      brand: { bg: "brand" },
      accent: { bg: "accent" },
    },
  },
  defaultVariants: { tone: "brand" },
});
