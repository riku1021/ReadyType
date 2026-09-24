import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const root = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "stretch",
  width: "100%",
  // 2つ目以降のセクション間だけ余白を確保（先頭は Header 直下なので空けない）
  "&:not(:first-child)": {
    paddingTop: "md",
  },
});

export const title = css({
  margin: "0",
  paddingTop: "0",
  paddingBottom: "10px",
  paddingInline: "sm",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const card = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  overflow: "hidden",
  marginInline: "0",
  // Drawer の drawerCard と同じ枠線＋影
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  borderRadius: "12px",
});

export const divider = css({
  height: "1px",
  margin: "0",
  border: "none",
  backgroundColor: "border.primary",
});
