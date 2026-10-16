import { css } from "@/styled-system/css";

export const appIconWrap = css({
  position: "relative",
  display: "inline-flex",
  width: "36px",
  height: "36px",
  flexShrink: "0",
});

export const appIconWrapLarge = css({
  position: "relative",
  display: "inline-flex",
  width: "56px",
  height: "56px",
  flexShrink: "0",
});

export const appIcon = css({
  width: "36px",
  height: "36px",
  borderRadius: "9px",
  flexShrink: "0",
  objectFit: "cover",
  display: "block",
});

export const appIconLarge = css({
  width: "56px",
  height: "56px",
  borderRadius: "13px",
  flexShrink: "0",
  objectFit: "cover",
  display: "block",
});

export const appIconFallback = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "36px",
  height: "36px",
  borderRadius: "9px",
  flexShrink: "0",
  backgroundColor: "background.sub",
  color: "text.secondary",
  fontWeight: "semibold",
  fontSize: "sm",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
});

export const appIconFallbackLarge = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  width: "56px",
  height: "56px",
  borderRadius: "13px",
  flexShrink: "0",
  backgroundColor: "background.sub",
  color: "text.secondary",
  fontWeight: "semibold",
  fontSize: "xl",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}",
});

/** 入力ソース欠損などを伝える警告バッジ（右上） */
export const appIconWarningBadge = css({
  position: "absolute",
  top: "-3px",
  right: "-3px",
  width: "16px",
  height: "16px",
  borderRadius: "full",
  backgroundColor: "status.error.main",
  color: "white",
  fontSize: "11px",
  fontWeight: "bold",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  lineHeight: "1",
  zIndex: "front",
  pointerEvents: "none",
});

export const select = css({
  paddingInline: "sm",
  paddingBlock: "xs",
  borderRadius: "8px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "border.primary",
  backgroundColor: "background.sub",
  color: "text.main",
  fontSize: "sm",
  cursor: "pointer",
});

export const selectWide = css({
  width: "100%",
  maxWidth: "360px",
});

/** セレクト横に警告バッジを並べる行 */
export const selectRow = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "xs",
  minWidth: "0",
});

export const selectRowWide = css({
  display: "flex",
  width: "100%",
  maxWidth: "384px",
});

/** バッジ付き wide 時: 行内で伸縮しつつ 360px を超えない */
export const selectWideInRow = css({
  flex: "1",
  minWidth: "0",
  maxWidth: "360px",
});

/** 入力ソース欠損などを伝えるインライン警告バッジ */
export const selectWarningBadge = css({
  width: "16px",
  height: "16px",
  borderRadius: "full",
  backgroundColor: "status.error.main",
  color: "white",
  fontSize: "11px",
  fontWeight: "bold",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  lineHeight: "1",
  flexShrink: "0",
  pointerEvents: "none",
});
