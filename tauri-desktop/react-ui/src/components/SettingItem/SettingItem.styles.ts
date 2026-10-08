import { css, cva } from "@/styled-system/css";

export const row = cva({
  base: {
    display: "flex",
    alignItems: "center",
    width: "100%",
    gap: "sm",
    paddingInline: "sm",
    border: "none",
    backgroundColor: "transparent",
    textAlign: "left",
    color: "text.main",
    font: "inherit",
    cursor: "pointer",
    transitionProperty: "background-color",
    transitionDuration: "150ms",
    transitionTimingFunction: "ease",
    _hover: {
      backgroundColor: "background.hover",
    },
    _focusVisible: {
      outline: "2px solid",
      outlineColor: "border.focus",
      outlineOffset: "-2px",
    },
  },
  variants: {
    density: {
      default: {
        paddingBlock: "sm",
      },
      compact: {
        paddingBlock: "10px",
      },
    },
    interactive: {
      true: {},
      false: {
        cursor: "default",
        _hover: {
          backgroundColor: "transparent",
        },
      },
    },
  },
  defaultVariants: {
    density: "default",
    interactive: true,
  },
});

export const icon = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  width: "24px",
  height: "24px",
  fontSize: "24px",
  color: "icon.primary",
  "& > svg": {
    width: "100%",
    height: "100%",
  },
});

export const iconError = css({
  color: "status.error.main",
});

export const textStack = css({
  display: "flex",
  flexDirection: "column",
  flex: "1",
  minWidth: "0",
  gap: "2px",
});

export const title = css({
  fontSize: "base",
  color: "text.main",
});

export const titleError = css({
  color: "status.error.main",
});

export const description = css({
  fontSize: "sm",
  color: "text.placeholder",
});

export const trailing = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "xs",
  flexShrink: "0",
  color: "icon.secondary",
});

export const value = css({
  fontSize: "base",
  color: "text.main",
});

export const chevron = css({
  display: "inline-flex",
  width: "24px",
  height: "24px",
  fontSize: "24px",
  color: "icon.secondary",
  "& > svg": {
    width: "100%",
    height: "100%",
  },
});
