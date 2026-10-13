import { css, cva } from "@/styled-system/css";

/** HTML モック（入力欄にルールを設定してみましょう.html / 800×600）に寄せた Quick Set 体験説明 */

export const root = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  width: "100%",
});

export const title = css({
  fontSize: "32px",
  fontWeight: "800",
  textAlign: "center",
  color: "#0d1330",
  lineHeight: "1.3",
  _dark: {
    color: "#e8eaf4",
  },
});

export const titleAccent = css({
  backgroundImage: "linear-gradient(90deg, #0a64ff, #8b4dff)",
  backgroundClip: "text",
  color: "transparent",
});

export const lead = css({
  marginTop: "6px",
  textAlign: "center",
  color: "#6b7190",
  fontSize: "13.5px",
  lineHeight: "1.6",
  fontWeight: "500",
  maxWidth: "40rem",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const chip = css({
  display: "inline-block",
  backgroundColor: "#e8f0ff",
  borderWidth: "1.5px",
  borderStyle: "solid",
  borderColor: "#bcd3ff",
  borderRadius: "8px",
  paddingBlock: "1px",
  paddingInline: "10px",
  color: "#4a6fd8",
  fontWeight: "600",
  letterSpacing: "0.2em",
  lineHeight: "1.4",
  _dark: {
    backgroundColor: "rgba(26, 109, 255, 0.2)",
    borderColor: "#3a5a9a",
    color: "#8eb6ff",
  },
});

export const demo = css({
  width: "fit-content",
  maxWidth: "100%",
  marginTop: "16px",
  marginInline: "auto",
  display: "grid",
  gridTemplateColumns: "auto auto auto",
  alignItems: "center",
  gap: "10px",
});

export const chat = css({
  backgroundColor: "#eef0fb",
  borderRadius: "10px",
  paddingTop: "5px",
  paddingBottom: "7px",
  paddingInline: "7px",
  boxShadow: "0 4px 14px rgba(80, 100, 200, 0.15)",
  textAlign: "start",
  _dark: {
    backgroundColor: "#222222",
  },
});

export const chatTraffic = css({
  display: "flex",
  gap: "6px",
  marginTop: "2px",
  marginBottom: "5px",
  marginInlineStart: "3px",
});

export const traffic = cva({
  base: {
    width: "9px",
    height: "9px",
    borderRadius: "full",
    flexShrink: "0",
  },
  variants: {
    tone: {
      red: { backgroundColor: "#ff5f57" },
      yellow: { backgroundColor: "#febc2e" },
      green: { backgroundColor: "#28c840" },
    },
    size: {
      sm: { width: "6px", height: "6px" },
      md: {},
    },
  },
  defaultVariants: {
    size: "md",
  },
});

export const chatInner = css({
  backgroundColor: "background.sub",
  borderRadius: "8px",
  paddingBlock: "7px",
  paddingInline: "10px",
});

export const chatTitle = css({
  display: "flex",
  alignItems: "center",
  gap: "7px",
  fontWeight: "700",
  fontSize: "12px",
  color: "#0d1330",
  marginBottom: "6px",
  _dark: {
    color: "#e8eaf4",
  },
});

export const chatBubbleIcon = css({
  width: "18px",
  height: "18px",
  borderRadius: "full",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "#1c2340",
  flexShrink: "0",
  _dark: {
    borderColor: "#c8cde0",
  },
});

export const chatField = css({
  backgroundColor: "white",
  borderRadius: "7px",
  paddingBlock: "9px",
  paddingInline: "6px",
  color: "#555a78",
  fontSize: "13px",
  boxShadow: "inset 0 0 0 1px #eef0f8",
  _dark: {
    backgroundColor: "#222222",
    color: "#a8aec8",
    boxShadow: "inset 0 0 0 1px #2a3050",
  },
});

export const caret = css({
  display: "inline-block",
  width: "3px",
  height: "15px",
  marginInlineStart: "6px",
  backgroundColor: "#0a64ff",
  borderRadius: "2px",
  verticalAlign: "middle",
});

export const keys = css({
  display: "flex",
  alignItems: "center",
  gap: "6px",
});

export const keysLead = css({
  width: "14px",
  borderTopWidth: "2px",
  borderTopStyle: "dotted",
  borderTopColor: "#a8aed0",
});

export const keyCap = css({
  width: "38px",
  height: "36px",
  borderRadius: "10px",
  backgroundColor: "background.sub",
  display: "grid",
  placeItems: "center",
  fontSize: "18px",
  fontWeight: "600",
  color: "#1a58e0",
  boxShadow: "0 4px 12px rgba(70, 100, 220, 0.2), inset 0 0 0 1px #e6ebfb",
  _dark: {
    color: "#8eb6ff",
    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.3), inset 0 0 0 1px #2a3050",
  },
});

export const keysChev = css({
  width: "8px",
  height: "8px",
  borderTopWidth: "2px",
  borderRightWidth: "2px",
  borderTopStyle: "solid",
  borderRightStyle: "solid",
  borderTopColor: "#8a90e0",
  borderRightColor: "#8a90e0",
  transform: "rotate(45deg)",
});

export const panel = css({
  backgroundColor: "background.sub",
  borderRadius: "14px",
  paddingTop: "12px",
  paddingBottom: "13px",
  paddingInline: "14px",
  boxShadow: "0 8px 30px rgba(70, 90, 200, 0.18)",
  textAlign: "start",
  _dark: {
    boxShadow: "0 8px 30px rgba(0, 0, 0, 0.35)",
  },
});

export const panelHeader = css({
  display: "flex",
  alignItems: "center",
  gap: "8px",
  fontWeight: "700",
  fontSize: "12px",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const panelLogo = css({
  width: "16px",
  height: "auto",
  objectFit: "contain",
  flexShrink: "0",
});

export const panelGear = css({
  marginInlineStart: "auto",
  color: "#7a80a0",
  fontSize: "14px",
  fontWeight: "400",
});

export const fieldLabel = css({
  marginTop: "8px",
  fontSize: "10px",
  color: "#555b7c",
  _dark: {
    color: "#a8aec8",
  },
});

export const fieldInput = css({
  marginTop: "4px",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "#dfe3f2",
  borderRadius: "7px",
  paddingBlock: "5px",
  paddingInline: "8px",
  fontSize: "12px",
  boxShadow: "0 2px 6px rgba(70, 90, 200, 0.08)",
  color: "#0d1330",
  _dark: {
    borderColor: "#2a3050",
    backgroundColor: "#12162a",
    color: "#e8eaf4",
  },
});

export const option = css({
  display: "flex",
  alignItems: "center",
  gap: "8px",
  marginTop: "7px",
  fontSize: "12px",
  fontWeight: "500",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const radio = cva({
  base: {
    width: "15px",
    height: "15px",
    borderRadius: "full",
    borderWidth: "1.5px",
    borderStyle: "solid",
    borderColor: "#cfd3e6",
    flexShrink: "0",
    boxSizing: "border-box",
  },
  variants: {
    on: {
      true: {
        borderWidth: "4.5px",
        borderColor: "#0a64ff",
        backgroundColor: "white",
      },
      false: {},
    },
  },
  defaultVariants: {
    on: false,
  },
});

export const sourceGlyph = cva({
  base: {
    width: "17px",
    height: "17px",
    borderRadius: "5px",
    display: "grid",
    placeItems: "center",
    color: "white",
    fontSize: "10px",
    fontWeight: "700",
    flexShrink: "0",
    textDecoration: "none",
  },
  variants: {
    tone: {
      primary: { backgroundColor: "#3a9bff" },
      english: { backgroundColor: "#8c5cf0" },
    },
  },
});

export const steps = css({
  width: "100%",
  maxWidth: "740px",
  marginTop: "18px",
  display: "grid",
  gridTemplateColumns: "1fr auto 1fr auto 1fr",
  alignItems: "start",
});

export const step = css({
  textAlign: "center",
});

export const stepTop = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "10px",
});

export const stepNum = css({
  width: "30px",
  height: "30px",
  borderRadius: "full",
  backgroundColor: "#0a64ff",
  color: "white",
  fontWeight: "700",
  display: "grid",
  placeItems: "center",
  fontSize: "15px",
  flexShrink: "0",
});

export const stepIcon = css({
  width: "44px",
  height: "44px",
  borderRadius: "full",
  backgroundColor: "#e6f0ff",
  display: "grid",
  placeItems: "center",
  color: "#1a7bff",
  fontSize: "20px",
  fontWeight: "600",
  flexShrink: "0",
  _dark: {
    backgroundColor: "rgba(26, 123, 255, 0.2)",
    color: "#8eb6ff",
  },
});

export const stepTitle = css({
  marginTop: "8px",
  fontSize: "15px",
  fontWeight: "800",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const stepBody = css({
  marginTop: "3px",
  color: "#6b7190",
  fontSize: "12px",
  lineHeight: "1.5",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const stepSep = css({
  width: "11px",
  height: "11px",
  borderTopWidth: "2px",
  borderRightWidth: "2px",
  borderTopStyle: "solid",
  borderRightStyle: "solid",
  borderTopColor: "#c3c8e8",
  borderRightColor: "#c3c8e8",
  transform: "rotate(45deg)",
  marginTop: "34px",
  marginInline: "4px",
});
