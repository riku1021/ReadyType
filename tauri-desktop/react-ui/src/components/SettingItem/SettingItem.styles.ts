import { css, cva } from "@/styled-system/css";

export const row = cva({
  base: {
    display: "flex",
    alignItems: "center",
    width: "100%",
    gap: "sm",
    paddingInline: "sm",
    // 右端アイコン行は中身 24px、トグル行はトラック 26px。上下 10px + minHeight でどちらも 46px にする
    boxSizing: "border-box",
    paddingBlock: "10px",
    minHeight: "52px",
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
    interactive: true,
  },
});

export const icon = cva({
  base: {
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
  },
  variants: {
    tone: {
      default: {},
      // cx だと atomic の順序で icon.primary が勝つことがあるため cva で上書きする
      danger: {
        color: "status.error.main",
      },
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

export const textStack = css({
  display: "flex",
  flexDirection: "column",
  flex: "1",
  minWidth: "0",
  gap: "0",
  position: "relative",
});

/** タイトルとサブ文を1つの塊にし、行の中央へ置く。塊は行の高さに含めない */
export const textGroup = css({
  position: "absolute",
  top: "50%",
  insetInline: "0",
  display: "flex",
  flexDirection: "column",
  gap: "2px",
  minWidth: "0",
  transform: "translateY(-50%)",
});

export const title = cva({
  base: {
    fontSize: "base",
    color: "text.main",
    lineHeight: "tight",
  },
  variants: {
    tone: {
      default: {},
      danger: {
        color: "status.error.main",
      },
    },
  },
  defaultVariants: {
    tone: "default",
  },
});

/** 1行を超える分は省略する */
export const description = css({
  overflow: "hidden",
  whiteSpace: "nowrap",
  textOverflow: "ellipsis",
  minWidth: "0",
  fontSize: "xs",
  color: "text.placeholder",
  lineHeight: "none",
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
