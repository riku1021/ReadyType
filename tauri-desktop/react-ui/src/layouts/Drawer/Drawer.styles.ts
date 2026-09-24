import { css } from "@/styled-system/css";

export const drawer = css({
  gridArea: "drawer",
  boxSizing: "border-box",
  padding: "sm",
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
  display: "flex",
  alignItems: "center",
  height: "60px",
  flexShrink: 0,
  textDecoration: "none",
  color: "text.main",
});

export const logoImageBox = css({
  width: "60px",
  height: "60px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: 0,
});

export const logoImage = css({
  height: "7",
  width: "auto",
});

export const logoText = css({
  display: "flex",
  alignItems: "center",
  whiteSpace: "nowrap",
  fontSize: "2xl",
  fontWeight: "semibold",
  lineHeight: "none",
});

export const logoAccent = css({
  color: "brand.main",
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

export const navText = css({
  whiteSpace: "nowrap",
  fontSize: "base",
});

export const themeToggleContainer = css({
  width: "60px",
  height: "60px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: 0,
});
