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
  flexShrink: "0",
  // ロゴ行は Drawer と同じ 60px。下の文言だけ横パディング
  paddingBottom: "sm",
  // 右上バツボタン（32px + sm）とロゴ文字が被らないよう余白を確保
  paddingRight: "48px",
});

export const appIdentity = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  minWidth: "0",
  paddingInline: "sm",
});

export const appName = css({
  margin: "0",
  minWidth: "0",
  fontSize: "base",
  fontWeight: "semibold",
  color: "text.main",
  overflow: "hidden",
  textOverflow: "ellipsis",
  whiteSpace: "nowrap",
});

export const prompt = css({
  margin: "0",
  paddingInline: "sm",
  fontSize: "xs",
  color: "text.secondary",
});

export const nameField = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  flexShrink: "0",
  paddingInline: "sm",
  paddingBottom: "sm",
});

/** 固定窓内で選択肢だけ縦スクロール */
export const optionsScroll = css({
  flex: "1",
  minHeight: "0",
  overflowY: "auto",
  overscrollBehavior: "contain",
});

export const footerBlock = css({
  flexShrink: "0",
});

export const nameLabel = css({
  margin: "0",
  fontSize: "xs",
  color: "text.secondary",
});

export const nameInput = css({
  width: "100%",
  boxSizing: "border-box",
  paddingBlock: "xs",
  paddingInline: "sm",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  borderRadius: "8px",
  backgroundColor: "background.main",
  color: "text.main",
  font: "inherit",
  fontSize: "base",
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
