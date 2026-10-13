import { css, cva } from "@/styled-system/css";

/** HTML モック（入力欄を検出するために.html / 800×600）に寄せた権限説明ステップ */

export const root = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  width: "100%",
});

export const hero = css({
  position: "relative",
  marginTop: "14px",
  width: "240px",
  maxWidth: "90%",
  backgroundImage: "linear-gradient(#f1f3fc, #eceffb)",
  borderRadius: "16px",
  paddingTop: "8px",
  paddingBottom: "14px",
  paddingInline: "16px",
  boxShadow: "0 8px 30px rgba(80, 120, 255, 0.22)",
  _dark: {
    backgroundImage: "none",
    backgroundColor: "#282828",
    boxShadow: "0 8px 30px rgba(0, 0, 0, 0.35)",
  },
});

export const heroTail = css({
  position: "absolute",
  left: "50%",
  bottom: "-10px",
  transform: "translateX(-50%)",
  width: "0",
  height: "0",
  borderLeftWidth: "10px",
  borderRightWidth: "10px",
  borderTopWidth: "10px",
  borderLeftStyle: "solid",
  borderRightStyle: "solid",
  borderTopStyle: "solid",
  borderLeftColor: "transparent",
  borderRightColor: "transparent",
  borderTopColor: "#b9d0ff",
  _dark: {
    borderTopColor: "#282828",
  },
});

export const trafficRow = css({
  display: "flex",
  gap: "6px",
  marginBottom: "6px",
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

export const askField = css({
  backgroundColor: "white",
  borderWidth: "3px",
  borderStyle: "solid",
  borderColor: "brand.main",
  borderRadius: "12px",
  paddingBlock: "8px",
  paddingInline: "14px",
  color: "#8a90ab",
  fontSize: "14px",
  boxShadow: "0 0 0 4px rgba(100, 170, 255, 0.2), 0 0 22px rgba(60, 140, 255, 0.35)",
  display: "flex",
  alignItems: "center",
  gap: "5px",
  _dark: {
    backgroundColor: "#222222",
    color: "#a8aec8",
  },
});

export const spark = css({
  position: "absolute",
  right: "-8px",
  top: "-2px",
  width: "36px",
  height: "40px",
  pointerEvents: "none",
});

export const sparkMark = cva({
  base: {
    position: "absolute",
    width: "4px",
    height: "12px",
    borderRadius: "3px",
    backgroundColor: "#2aa4ff",
  },
  variants: {
    index: {
      "0": { left: "0", top: "0" },
      "1": { left: "14px", top: "8px", transform: "rotate(45deg)" },
      "2": { left: "22px", top: "26px", width: "12px", height: "4px" },
    },
  },
});

export const title = css({
  marginTop: "20px",
  fontSize: "32px",
  fontWeight: "800",
  textAlign: "center",
  color: "#0d1330",
  lineHeight: "1.3",
  letterSpacing: "-0.01em",
  _dark: {
    color: "#e8eaf4",
  },
});

export const titleAccent = css({
  color: "#0a64ff",
  _dark: {
    color: "#5b9cff",
  },
});

export const lead = css({
  marginTop: "6px",
  textAlign: "center",
  color: "#6b7190",
  fontSize: "14px",
  lineHeight: "1.6",
  fontWeight: "500",
  maxWidth: "40rem",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const flow = css({
  width: "100%",
  marginTop: "18px",
  display: "grid",
  gridTemplateColumns: "1.15fr auto auto auto 1.2fr",
  gap: "8px",
  alignItems: "center",
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

export const arrow = css({
  display: "block",
  width: "34px",
  height: "0",
  borderTopWidth: "2px",
  borderTopStyle: "dotted",
  borderTopColor: "#a8aed0",
  position: "relative",
  justifySelf: "center",
  marginInline: "6px",
  _after: {
    content: '""',
    position: "absolute",
    right: "0",
    top: "-6px",
    width: "9px",
    height: "9px",
    borderTopWidth: "2px",
    borderRightWidth: "2px",
    borderTopStyle: "solid",
    borderRightStyle: "solid",
    borderTopColor: "#8a90b8",
    borderRightColor: "#8a90b8",
    transform: "rotate(45deg)",
  },
});

export const center = css({
  textAlign: "center",
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
});

export const centerLogo = css({
  width: "46px",
  height: "auto",
  objectFit: "contain",
});

export const centerName = css({
  display: "block",
  fontSize: "16px",
  fontWeight: "700",
  marginTop: "4px",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const focusBadge = css({
  display: "inline-block",
  marginTop: "6px",
  backgroundColor: "#e1eeff",
  color: "#1a6dff",
  borderRadius: "full",
  paddingBlock: "4px",
  paddingInline: "12px",
  fontSize: "12px",
  fontWeight: "500",
  _dark: {
    backgroundColor: "rgba(26, 109, 255, 0.25)",
    color: "#8eb6ff",
  },
});

export const resultCard = css({
  backgroundColor: "background.sub",
  borderRadius: "16px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
  paddingBlock: "16px",
  paddingInline: "14px",
  textAlign: "center",
});

export const langPill = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "10px",
  backgroundColor: "background.sub",
  borderRadius: "full",
  paddingTop: "7px",
  paddingBottom: "7px",
  paddingLeft: "7px",
  paddingRight: "28px",
  boxShadow: "0 4px 14px rgba(80, 100, 200, 0.14), inset 0 0 0 1px #eef0f8",
  fontSize: "19px",
  fontWeight: "700",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
    boxShadow: "0 4px 14px rgba(0, 0, 0, 0.25), inset 0 0 0 1px #2a3050",
  },
});

export const langGlyph = css({
  width: "36px",
  height: "36px",
  borderRadius: "full",
  backgroundImage: "linear-gradient(#3aa0ff, #1a6dff)",
  color: "white",
  display: "inline-flex",
  alignItems: "center",
  justifyContent: "center",
  fontSize: "20px",
  fontWeight: "700",
  flexShrink: "0",
});

export const resultNote = css({
  marginTop: "8px",
  color: "#6b7190",
  fontSize: "12.5px",
  lineHeight: "1.6",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const privacy = css({
  width: "100%",
  marginTop: "16px",
  display: "flex",
  alignItems: "center",
  gap: "20px",
  paddingBlock: "12px",
  paddingInline: "22px",
  backgroundColor: "background.sub",
  borderRadius: "16px",
  boxShadow: "0 8px 28px rgba(70, 90, 200, 0.1)",
  textAlign: "start",
  _dark: {
    boxShadow: "0 8px 28px rgba(0, 0, 0, 0.3)",
  },
});

export const lockIcon = css({
  flexShrink: "0",
  width: "56px",
  height: "56px",
  borderRadius: "full",
  backgroundColor: "status.info.main",
  color: "white",
  display: "grid",
  placeItems: "center",
  lineHeight: "1",
  fontSize: "22px",
  "& > svg": {
    display: "block",
    width: "22px",
    height: "22px",
  },
});

export const privacyCopy = css({
  minInlineSize: "0",
  flex: "1",
});

export const privacyTitle = css({
  fontSize: "15px",
  fontWeight: "700",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const privacyBody = css({
  marginTop: "3px",
  color: "#6b7190",
  fontSize: "12.5px",
  lineHeight: "1.6",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});
