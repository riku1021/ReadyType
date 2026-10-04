import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

/** 窓そのものがセクションカード（外側の板・余白なし） */
export const shell = css(surfaceSub, {
  position: "relative",
  display: "flex",
  flexDirection: "column",
  height: "100vh",
  width: "100%",
  overflow: "hidden",
  boxSizing: "border-box",
  color: "text.main",
  userSelect: "none",
  // SettingsSection.card と同じ枠線＋影＋角丸
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  borderRadius: "12px",
});

/** Flutter SelectionBottomModal の円形 ✗ に準拠 */
export const closeButton = css({
  position: "absolute",
  top: "sm",
  right: "sm",
  zIndex: "front",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "32px",
  height: "32px",
  padding: "0",
  borderWidth: "0",
  borderRadius: "999px",
  backgroundColor: "background.hover",
  color: "icon.primary",
  cursor: "pointer",
  fontSize: "24px",
  lineHeight: "1",
  transitionProperty: "background-color, color",
  transitionDuration: "150ms",
  transitionTimingFunction: "ease",
  _hover: {
    color: "status.error.main",
  },
  _focusVisible: {
    outline: "2px solid",
    outlineColor: "border.focus",
    outlineOffset: "2px",
  },
  _disabled: {
    opacity: 0.5,
    cursor: "not-allowed",
  },
});

export const divider = css({
  height: "1px",
  margin: "0",
  border: "none",
  flexShrink: "0",
  backgroundColor: "border.primary",
});

export const headerBlock = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  paddingBlock: "sm",
  paddingInline: "sm",
  // 右上バツボタン（32px + sm）と被らないよう余白を確保
  paddingRight: "48px",
});

export const brand = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const targetLabel = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const prompt = css({
  margin: "0",
  fontSize: "xs",
  color: "text.secondary",
});

export const optionButton = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "sm",
  width: "100%",
  paddingBlock: "10px",
  paddingInline: "sm",
  border: "none",
  borderRadius: "0",
  backgroundColor: "transparent",
  color: "text.main",
  font: "inherit",
  textAlign: "left",
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
  _disabled: {
    opacity: 0.5,
    cursor: "not-allowed",
  },
});

export const optionButtonCurrent = css({
  backgroundColor: "background.hover",
});

export const optionText = css({
  display: "flex",
  flexDirection: "column",
  gap: "2px",
  minWidth: "0",
  flex: "1",
});

export const radio = css({
  width: "16px",
  height: "16px",
  marginTop: "2px",
  borderRadius: "999px",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "border.primary",
  flexShrink: "0",
  boxSizing: "border-box",
});

export const radioChecked = css({
  borderColor: "brand.main",
  backgroundColor: "brand.main",
  boxShadow: "inset 0 0 0 3px {colors.background.sub}",
});

export const optionTitle = css({
  fontSize: "base",
  color: "text.main",
});

export const optionSubtitle = css({
  fontSize: "xs",
  color: "text.secondary",
});

export const emptyState = css({
  margin: "0",
  paddingBlock: "sm",
  paddingInline: "sm",
  fontSize: "sm",
  color: "text.secondary",
});
