import { css, cva } from "@/styled-system/css";

/**
 * 共有ボタン recipe。
 * - サイズは共通 1 種（padding / fontSize）
 * - 塗り primary: hover は brand.hover
 * - 枠系 secondary / danger / dashed: 枠幅は親面の --button-framed-border-width
 * - neutral: icon.primary + 面 hover。枠 1px
 * - 影色は親面の --button-surface-shadow（styles/objects/surface.ts）
 * - primary の枠は親面の --button-primary-border-width（main 上で 1px）
 * - layout icon: アイコンのみ（横パディングを縦に合わせて正方形寄りに）
 * - 字重は共通で semibold
 */
export const button = cva({
  base: {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    gap: "xs",
    textDecoration: "none",
    fontWeight: "semibold",
    fontSize: "sm",
    paddingBlock: "xs",
    paddingInline: "sm",
    whiteSpace: "nowrap",
    minWidth: "0",
    borderWidth: "0",
    borderStyle: "none",
    cursor: "pointer",
    boxSizing: "border-box",
    boxShadow: "0 2px 8px var(--button-surface-shadow)",
    transitionProperty: "background-color, border-color, color",
    transitionDuration: "500ms",
    transitionTimingFunction: "ease",
    _disabled: {
      opacity: 0.5,
      cursor: "not-allowed",
    },
  },
  variants: {
    variant: {
      primary: {
        backgroundColor: "brand.main",
        color: "white",
        borderRadius: "50px",
        borderWidth: "var(--button-primary-border-width)",
        borderStyle: "solid",
        borderColor: "brand.main",
        _hover: {
          backgroundColor: "brand.hover",
          borderColor: "brand.hover",
        },
      },
      secondary: {
        backgroundColor: "background.sub",
        color: "brand.main",
        borderWidth: "var(--button-framed-border-width)",
        borderStyle: "solid",
        borderColor: "brand.main",
        borderRadius: "50px",
        _hover: {
          backgroundColor: "background.main",
        },
      },
      danger: {
        backgroundColor: "background.sub",
        color: "status.error.main",
        borderWidth: "var(--button-framed-border-width)",
        borderStyle: "solid",
        borderColor: "status.error.main",
        borderRadius: "50px",
        _hover: {
          backgroundColor: "background.main",
        },
      },
      neutral: {
        backgroundColor: "background.sub",
        color: "icon.primary",
        borderWidth: "1px",
        borderStyle: "solid",
        borderColor: "border.primary",
        borderRadius: "50px",
        _hover: {
          backgroundColor: "background.main",
        },
      },
      dashed: {
        backgroundColor: "transparent",
        color: "brand.main",
        borderWidth: "var(--button-framed-border-width)",
        borderStyle: "dashed",
        borderColor: "brand.main",
        borderRadius: "50px",
        _hover: {
          backgroundColor: "background.main",
        },
      },
      text: {
        backgroundColor: "transparent",
        color: "brand.main",
        borderRadius: "50px",
        boxShadow: "none",
        _hover: {
          color: "brand.hover",
          textDecoration: "underline",
        },
      },
    },
    layout: {
      labeled: {},
      icon: {
        paddingInline: "xs",
      },
    },
    fullWidth: {
      true: {
        width: "100%",
      },
    },
  },
  defaultVariants: {
    variant: "primary",
    layout: "labeled",
  },
});

export type ButtonVariant = "primary" | "secondary" | "danger" | "neutral" | "dashed" | "text";

export type ButtonLayout = "labeled" | "icon";

/**
 * 左右アイコン枠。
 * ボタンの fontSize に追従する（compact の xs など）ため rem 固定ではなく em にする。
 */
export const icon = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  width: "1.25em",
  height: "1.25em",
  "& > svg": {
    width: "100%",
    height: "100%",
  },
});
