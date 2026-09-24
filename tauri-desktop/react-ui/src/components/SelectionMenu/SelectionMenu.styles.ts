import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const panel = css(surfaceSub, {
  position: "fixed",
  zIndex: "floating",
  minWidth: "160px",
  maxHeight: "min(360px, calc(100vh - 24px))",
  overflowY: "auto",
  paddingBlock: "xs",
  // Drawer / SettingsSection と同じ枠線＋影
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  borderRadius: "12px",
});

export const list = css({
  display: "flex",
  flexDirection: "column",
  margin: "0",
  padding: "0",
  listStyle: "none",
});

export const option = css({
  display: "block",
  width: "100%",
  paddingBlock: "10px",
  paddingInline: "md",
  border: "none",
  borderRadius: "0",
  backgroundColor: "transparent",
  color: "text.main",
  font: "inherit",
  fontSize: "base",
  textAlign: "left",
  whiteSpace: "nowrap",
  cursor: "pointer",
  transitionProperty: "background-color, color",
  transitionDuration: "150ms",
  transitionTimingFunction: "ease",
  _hover: {
    backgroundColor: "brand.main",
    color: "white",
  },
  _focusVisible: {
    outline: "2px solid",
    outlineColor: "border.focus",
    outlineOffset: "-2px",
  },
});

export const optionSelected = css({
  backgroundColor: "background.hover",
});
