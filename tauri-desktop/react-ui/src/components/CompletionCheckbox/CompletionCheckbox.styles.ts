import { css, cva } from "@/styled-system/css";

export const control = css({
  position: "relative",
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "1.15rem",
  height: "1.15rem",
  flexShrink: "0",
  cursor: "pointer",
});

export const input = css({
  position: "absolute",
  inset: "0",
  zIndex: "1",
  width: "100%",
  height: "100%",
  margin: "0",
  opacity: "0",
  cursor: "pointer",
});

export const icon = cva({
  base: {
    width: "1.15rem",
    height: "1.15rem",
    flexShrink: "0",
    pointerEvents: "none",
  },
  variants: {
    completed: {
      true: {
        color: "brand.main",
      },
      false: {
        color: "border.primary",
      },
    },
  },
  defaultVariants: {
    completed: false,
  },
});
