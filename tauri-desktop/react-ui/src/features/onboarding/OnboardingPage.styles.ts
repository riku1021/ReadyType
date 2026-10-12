import { css, cva } from "@/styled-system/css";
import { surfaceMain, surfaceSub } from "@/styles/objects/surface";

export const page = css(surfaceMain, {
  display: "flex",
  flexDirection: "column",
  minHeight: "100vh",
  height: "100vh",
  boxSizing: "border-box",
  paddingTop: "12px",
  paddingBottom: "16px",
  paddingInline: "28px",
});

export const progressBlock = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  flexShrink: "0",
});

/** 進捗とフッターのあいだ。枠はウィンドウに追従し、中身の高さはコンテンツ次第 */
export const content = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "stretch",
  justifyContent: "center",
  flex: "1",
  minHeight: "0",
  width: "100%",
});

export const progressTrack = css({
  display: "flex",
  alignItems: "center",
  gap: "10px",
});

export const progressDot = cva({
  base: {
    width: "10px",
    height: "10px",
    borderRadius: "full",
    backgroundColor: "#d9dbe6",
    flexShrink: "0",
    _dark: {
      backgroundColor: "#3a3f55",
    },
  },
  variants: {
    state: {
      done: { backgroundColor: "#0a64ff" },
      current: { backgroundColor: "#0a64ff" },
      upcoming: {},
    },
  },
  defaultVariants: {
    state: "upcoming",
  },
});

export const infoCard = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  padding: "md",
  borderRadius: "12px",
  textAlign: "start",
  maxWidth: "28rem",
  width: "100%",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
});

export const infoRow = css({
  display: "flex",
  alignItems: "flex-start",
  gap: "sm",
});

export const infoIcon = css({
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: "0",
  width: "36px",
  height: "36px",
  borderRadius: "10px",
  fontSize: "18px",
  color: "white",
  backgroundColor: "brand.main",
});

export const infoTextStack = css({
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  minInlineSize: "0",
  flex: "1",
});

export const infoTitle = css({
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const infoBody = css({
  fontSize: "xs",
  color: "text.sub",
  lineHeight: "1.5",
});

export const previewGrid = css({
  display: "flex",
  flexWrap: "wrap",
  justifyContent: "center",
  gap: "sm",
  maxWidth: "28rem",
  width: "100%",
});

export const previewCard = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "xs",
  padding: "sm",
  borderRadius: "10px",
  width: "8.5rem",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  textAlign: "start",
});

export const previewTitle = css({
  fontSize: "xs",
  fontWeight: "semibold",
  color: "text.main",
});

export const previewBadge = cva({
  base: {
    alignSelf: "flex-start",
    fontSize: "xs",
    fontWeight: "semibold",
    paddingInline: "xs",
    paddingBlock: "xs",
    borderRadius: "full",
    color: "white",
  },
  variants: {
    tone: {
      english: { backgroundColor: "brand.main" },
      japanese: { backgroundColor: "brand.hover" },
    },
  },
});

export const footer = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "stretch",
  gap: "12px",
  flexShrink: "0",
  width: "100%",
  marginTop: "auto",
  paddingTop: "14px",
});

export const footerRow = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "14px",
  width: "100%",
});

export const footerInfo = css({
  width: "100%",
  backgroundColor: "#edf3ff",
  borderRadius: "12px",
  paddingBlock: "9px",
  paddingInline: "14px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "9px",
  color: "#3a4a8a",
  fontSize: "11.5px",
  fontWeight: "500",
  textAlign: "start",
  _dark: {
    backgroundColor: "rgba(26, 109, 255, 0.15)",
    color: "#b4c4f0",
  },
});

export const footerInfoIcon = css({
  flexShrink: "0",
  color: "#1a6dff",
  fontSize: "18px",
  _dark: {
    color: "#8eb6ff",
  },
});

export const footerSpacer = css({
  width: "1px",
  flex: "1",
});

export const footerActions = css({
  display: "flex",
  alignItems: "center",
  gap: "sm",
  marginInlineStart: "auto",
});

export const footerPrimaryColumn = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "stretch",
  gap: "8px",
  flex: "1",
  maxWidth: "520px",
  marginInlineStart: "auto",
  textAlign: "center",
});

export const footerNote = css({
  marginTop: "0",
  fontSize: "11px",
  color: "#6b7190",
  textAlign: "center",
  lineHeight: "1.45",
  _dark: {
    color: "#a8aec8",
  },
});
