import { css } from "@/styled-system/css";

export const drawer = css({
  gridArea: "drawer",
  boxSizing: "border-box",
  paddingInline: "sm",
  paddingBottom: "sm",
  paddingTop: "0",
  height: "100%",
  minHeight: "0",
});

export const drawerCard = css({
  boxSizing: "border-box",
  width: "200px",
  height: "100%",
  display: "flex",
  flexDirection: "column",
  backgroundColor: "background.sub",
  "--button-surface-shadow": "{colors.shadow.secondary}",
  "--button-framed-border-width": "2px",
  "--button-primary-border-width": "0px",
  // border だと content 幅がずれるため inset boxShadow で枠線を表現
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  borderRadius: "3xl",
  overflow: "hidden",
});

export const logoLink = css({
  display: "block",
  textDecoration: "none",
  color: "text.main",
  flexShrink: 0,
});

export const navLinks = css({
  display: "flex",
  flexDirection: "column",
  flex: "1",
  overflowX: "hidden",
  overflowY: "auto",
});

export const navLink = css({
  color: "text.sub",
  textDecoration: "none",
  fontSize: "base",
  display: "block",
  position: "relative",
  // 項目内の余白（青ハイライトの高さ）。inset より大きくして枠内に余白を残す
  paddingBlock: "10px",
  _hover: {
    "& [data-part=highlight]": {
      backgroundColor: "background.hover",
    },
  },
  _currentPage: {
    color: "white",
    "& [data-part=highlight]": {
      backgroundColor: "brand.main",
    },
    _hover: {
      "& [data-part=highlight]": {
        backgroundColor: "brand.main",
      },
    },
  },
});

/** ホバー面（Button と同様に background-color をトランジション） */
export const navLinkHighlight = css({
  position: "absolute",
  // inset を小さくして、行パディング分が青枠の内側余白になるようにする
  inset: "4px",
  borderRadius: "full",
  backgroundColor: "transparent",
  transitionProperty: "background-color",
  transitionDuration: "500ms",
  transitionTimingFunction: "ease",
  pointerEvents: "none",
});

export const navLinkContent = css({
  display: "flex",
  alignItems: "center",
  position: "relative",
  zIndex: "default",
});

export const navIconBox = css({
  width: "60px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  boxSizing: "border-box",
  flexShrink: 0,
});

export const navIcon = css({
  display: "block",
  width: "24px",
  height: "24px",
  fontSize: "24px",
  flexShrink: 0,
});

/** アイコン上にバッジを重ねるための相対ボックス */
export const navIconWrap = css({
  position: "relative",
  display: "inline-flex",
  width: "24px",
  height: "24px",
  flexShrink: 0,
});

/** 入力ソース欠損などを伝える警告バッジ（右上） */
export const navIconWarningBadge = css({
  position: "absolute",
  top: "-3px",
  right: "-3px",
  width: "14px",
  height: "14px",
  borderRadius: "full",
  backgroundColor: "status.error.main",
  color: "white",
  fontSize: "10px",
  fontWeight: "bold",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  lineHeight: "1",
  zIndex: "front",
  pointerEvents: "none",
});

export const navText = css({
  whiteSpace: "nowrap",
  fontSize: "base",
});

export const controlsRow = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  height: "60px",
  paddingInline: "sm",
  flexShrink: 0,
});
