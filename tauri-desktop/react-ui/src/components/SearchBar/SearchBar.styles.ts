import { css, cva } from "@/styled-system/css";

export const root = cva({
  base: {
    position: "relative",
    display: "flex",
    alignItems: "center",
    minWidth: "0",
    maxWidth: "22rem",
    width: "16rem",
  },
  variants: {
    fullWidth: {
      true: {
        flex: "1",
        width: "100%",
        maxWidth: "none",
      },
    },
  },
});

export const input = css({
  width: "100%",
  minWidth: "0",
  display: "block",
  color: "text.main",
  backgroundColor: "background.sub",
  borderRadius: "full",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  boxShadow: "0 2px 8px {colors.shadow.primary}",
  paddingBlock: "xs",
  paddingLeft: "sm",
  paddingRight: "2.5rem",
  fontSize: "sm",
  outline: "none",
  _placeholder: {
    color: "text.secondary",
  },
  _focus: {
    borderColor: "brand.main",
  },
  "&::-webkit-search-cancel-button": {
    WebkitAppearance: "none",
    appearance: "none",
    display: "none",
  },
  "&::-webkit-search-decoration": {
    WebkitAppearance: "none",
    appearance: "none",
  },
});

const affix = css.raw({
  position: "absolute",
  right: "0.35rem",
  top: "50%",
  transform: "translateY(-50%)",
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  width: "1.75rem",
  height: "1.75rem",
  borderRadius: "full",
  flexShrink: "0",
});

export const searchIcon = css(affix, {
  backgroundColor: "brand.main",
  color: "white",
  pointerEvents: "none",
});

export const clearButton = css(affix, {
  borderWidth: "0",
  padding: "0",
  backgroundColor: "text.placeholder",
  color: "white",
  cursor: "pointer",
  _hover: {
    opacity: "0.85",
  },
});

export const searchGlyph = css({
  width: "0.75rem",
  height: "0.75rem",
});

/** IoIosClose は viewBox 内の線が細いので、見た目を検索アイコンに合わせる */
export const clearGlyph = css({
  width: "1.25rem",
  height: "1.25rem",
});
