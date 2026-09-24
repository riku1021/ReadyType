import { css, cva } from "@/styled-system/css";

export const track = cva({
  base: {
    position: "relative",
    display: "inline-flex",
    alignItems: "center",
    width: "44px",
    height: "26px",
    padding: "0",
    border: "none",
    borderRadius: "999px",
    cursor: "pointer",
    flexShrink: "0",
    transitionProperty: "background-color",
    transitionDuration: "200ms",
    transitionTimingFunction: "ease",
    _disabled: {
      opacity: 0.5,
      cursor: "not-allowed",
    },
    _focusVisible: {
      outline: "2px solid",
      outlineColor: "border.focus",
      outlineOffset: "2px",
    },
  },
  variants: {
    checked: {
      true: {
        backgroundColor: "brand.main",
      },
      false: {
        backgroundColor: "background.main",
      },
    },
  },
  defaultVariants: {
    checked: false,
  },
});

export const thumb = css({
  position: "absolute",
  top: "2px",
  left: "2px",
  width: "22px",
  height: "22px",
  borderRadius: "999px",
  backgroundColor: "white",
  boxShadow: "0 1px 3px token(colors.shadow.primary)",
  transitionProperty: "transform",
  transitionDuration: "200ms",
  transitionTimingFunction: "ease",
  pointerEvents: "none",
});

export const thumbChecked = css({
  transform: "translateX(18px)",
});
