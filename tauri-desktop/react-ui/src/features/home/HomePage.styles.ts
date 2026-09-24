import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  maxWidth: "640px",
});

export const banner = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  padding: "sm",
  borderRadius: "12px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
});

export const bannerTitle = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const bannerText = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
});

export const bannerActions = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "xs",
});

export const card = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  padding: "sm",
  borderRadius: "12px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
});

export const cardHeader = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "sm",
});

export const headline = css({
  margin: "0",
  fontSize: "md",
  fontWeight: "semibold",
  color: "text.main",
});

export const row = css({
  display: "flex",
  justifyContent: "space-between",
  gap: "sm",
  fontSize: "sm",
});

export const label = css({
  color: "text.secondary",
});

export const value = css({
  color: "text.main",
  textAlign: "right",
  wordBreak: "break-all",
});
