import { css, cva } from "@/styled-system/css";

export const block = css({
  display: "flex",
  flexDirection: "column",
  gap: "0",
});

export const errorMessage = css({
  marginInline: "sm",
  marginBlockEnd: "xs",
  fontSize: "sm",
  color: "status.error.main",
});

export const modalError = css({
  marginBlockEnd: "sm",
  fontSize: "sm",
  color: "status.error.main",
});

export const row = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
  width: "100%",
  paddingInline: "sm",
  // SettingItem の行高 52px に揃える。右端のチップとリセットは 32px なので上下は 10px
  boxSizing: "border-box",
  paddingBlock: "10px",
  minHeight: "52px",
});

export const title = css({
  fontSize: "base",
  color: "text.main",
  minWidth: "0",
});

export const trailing = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "xs",
  flexShrink: "0",
});

export const chordChip = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  boxSizing: "border-box",
  height: "32px",
  paddingInline: "sm",
  paddingBlock: "0",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.hover",
  color: "text.main",
  fontSize: "sm",
  fontWeight: "semibold",
  fontFamily: "inherit",
  lineHeight: "1",
  cursor: "pointer",
  _hover: {
    backgroundColor: "background.sub",
  },
  _focusVisible: {
    outline: "2px solid",
    outlineColor: "border.focus",
    outlineOffset: "2px",
  },
});

export const iconButton = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "32px",
  height: "32px",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.hover",
  color: "text.main",
  fontSize: "md",
  cursor: "pointer",
  _hover: {
    backgroundColor: "background.sub",
  },
  _focusVisible: {
    outline: "2px solid",
    outlineColor: "border.focus",
    outlineOffset: "2px",
  },
  _disabled: {
    opacity: 0.45,
    cursor: "not-allowed",
  },
});

export const modifierRow = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "100%",
});

/** 記録中の修飾キーと本体キーを載せる一つの面 */
export const recorderPlate = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "100%",
  minInlineSize: "0",
  flexWrap: "wrap",
  // 修飾キー群と本体キーのあいだをアイコン1つ分空ける
  gap: "3rem",
  boxSizing: "border-box",
  paddingInline: "lg",
  paddingBlock: "sm",
  borderRadius: "16px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.hover",
});

export const modifierGroup = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "md",
});

export const modifierGlyph = cva({
  base: {
    display: "inline-flex",
    alignItems: "center",
    justifyContent: "center",
    minWidth: "1.25rem",
    fontSize: "xl",
    lineHeight: "1",
    userSelect: "none",
  },
  variants: {
    active: {
      true: {
        color: "brand.main",
      },
      false: {
        color: "text.placeholder",
      },
    },
    presentation: {
      symbol: {
        fontSize: "xl",
      },
      label: {
        fontSize: "xs",
        fontWeight: "semibold",
        minWidth: "2.25rem",
      },
    },
  },
  defaultVariants: {
    active: false,
    presentation: "symbol",
  },
});

/** ひし形／本体キーで幅が変わらないよう固定スロット */
export const keySlot = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  width: "1.75rem",
  height: "1.75rem",
  userSelect: "none",
});

export const keyDiamond = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "1.25rem",
  height: "1.25rem",
  borderRadius: "4px",
  transform: "rotate(45deg)",
  boxShadow: "inset 0 0 0 1.5px currentColor",
  color: "text.placeholder",
  fontSize: "sm",
  fontWeight: "semibold",
  lineHeight: "1",
});

export const keyDiamondInner = css({
  transform: "rotate(-45deg)",
});

export const keyGlyph = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "100%",
  height: "100%",
  color: "brand.main",
  fontSize: "xl",
  fontWeight: "semibold",
  lineHeight: "1",
});

export const actions = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "sm",
  width: "100%",
});
