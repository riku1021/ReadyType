import { css } from "@/styled-system/css";
import { surfaceSub } from "@/styles/objects/surface";

export const page = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
  maxWidth: "720px",
});

export const toolbar = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "xs",
});

export const list = css({
  display: "flex",
  flexDirection: "column",
  gap: "sm",
});

export const card = css(surfaceSub, {
  display: "flex",
  flexDirection: "column",
  gap: "xs",
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

export const title = css({
  margin: "0",
  fontSize: "sm",
  fontWeight: "semibold",
  color: "text.main",
});

export const meta = css({
  margin: "0",
  fontSize: "sm",
  color: "text.secondary",
  wordBreak: "break-all",
});

export const actions = css({
  display: "flex",
  flexWrap: "wrap",
  gap: "xs",
});
