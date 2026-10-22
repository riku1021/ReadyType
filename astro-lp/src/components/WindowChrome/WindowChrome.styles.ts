import { css, cva } from "styled-system/css";

export const windowFrame = css({
  bg: "white",
  borderRadius: "window",
  boxShadow: "card",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "line",
  position: "relative",
});

export const windowBar = css({
  display: "flex",
  gap: "6px",
  pt: "14px",
  px: "16px",
});

export const windowDot = cva({
  base: {
    width: "11px",
    height: "11px",
    borderRadius: "50%",
  },
  variants: {
    tone: {
      close: { bg: "#FF5F57" },
      min: { bg: "#FEBC2E" },
      max: { bg: "#28C840" },
    },
  },
});
