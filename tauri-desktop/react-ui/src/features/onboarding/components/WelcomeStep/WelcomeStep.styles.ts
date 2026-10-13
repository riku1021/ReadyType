import { css, cva } from "@/styled-system/css";

/** HTML モック（ReadyType へようこそ.html / 800×600）に寄せた Welcome */

export const root = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  width: "100%",
});

export const logo = css({
  marginTop: "12px",
  width: "64px",
  height: "auto",
  objectFit: "contain",
  flexShrink: "0",
});

export const title = css({
  marginTop: "10px",
  fontSize: "38px",
  fontWeight: "800",
  letterSpacing: "-0.01em",
  textAlign: "center",
  color: "#0d1330",
  lineHeight: "1.2",
  _dark: {
    color: "#e8eaf4",
  },
});

export const titleAccent = css({
  backgroundImage: "linear-gradient(90deg, #0a64ff, #7b4dff)",
  backgroundClip: "text",
  color: "transparent",
});

export const lead = css({
  marginTop: "6px",
  textAlign: "center",
  color: "#6b7190",
  fontSize: "15px",
  lineHeight: "1.6",
  fontWeight: "500",
  whiteSpace: "pre-line",
  maxWidth: "36rem",
  _dark: {
    color: "#a8aec8",
  },
});

export const demo = css({
  position: "relative",
  width: "100%",
  marginTop: "22px",
  display: "grid",
  gridTemplateColumns: "1fr 150px 1fr",
  alignItems: "center",
  gap: "10px",
});

export const demoCard = cva({
  base: {
    position: "relative",
    height: "150px",
    backgroundColor: "background.sub",
    borderRadius: "16px",
    paddingTop: "12px",
    paddingBottom: "12px",
    paddingInline: "16px",
    textAlign: "start",
    boxSizing: "border-box",
  },
  variants: {
    side: {
      message: {
        boxShadow: "0 8px 36px rgba(50, 150, 255, 0.22)",
      },
    },
  },
});

/** 権限画面 AI Chat と同じ外枠 + 内側カードのブラウザデモ */
export const browserWin = css({
  position: "relative",
  height: "150px",
  boxSizing: "border-box",
  backgroundColor: "#eef0fb",
  borderRadius: "10px",
  paddingTop: "5px",
  paddingBottom: "7px",
  paddingInline: "7px",
  boxShadow: "0 8px 36px rgba(150, 100, 255, 0.2)",
  textAlign: "start",
  display: "flex",
  flexDirection: "column",
  _dark: {
    backgroundColor: "#222222",
  },
});

export const browserTraffic = css({
  display: "flex",
  gap: "6px",
  marginTop: "2px",
  marginBottom: "5px",
  marginInlineStart: "3px",
  flexShrink: "0",
});

export const browserInner = css({
  backgroundColor: "background.sub",
  borderRadius: "8px",
  paddingBlock: "10px",
  paddingInline: "10px",
  flex: "1",
  minHeight: "0",
  display: "flex",
  alignItems: "center",
});

export const appRow = css({
  display: "flex",
  alignItems: "center",
  gap: "10px",
  fontWeight: "700",
  fontSize: "13px",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const messageAppIcon = css({
  width: "34px",
  height: "34px",
  borderRadius: "10px",
  backgroundColor: "#2fd05b",
  display: "grid",
  placeItems: "center",
  color: "white",
  fontSize: "16px",
  lineHeight: "1",
  flexShrink: "0",
  "& > svg": {
    display: "block",
    width: "16px",
    height: "16px",
  },
});

export const messageField = css({
  marginTop: "10px",
  backgroundColor: "white",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "#f0f1f8",
  borderRadius: "12px",
  padding: "9px",
  height: "62px",
  boxSizing: "border-box",
  _dark: {
    backgroundColor: "#12162a",
    borderColor: "#2a3050",
  },
});

export const messageBubble = css({
  display: "inline-block",
  backgroundColor: "#eceef6",
  borderRadius: "full",
  paddingBlock: "6px",
  paddingInline: "14px",
  color: "#555a78",
  fontSize: "13px",
  _dark: {
    backgroundColor: "#252a42",
    color: "#a8aec8",
  },
});

export const caret = css({
  display: "inline-block",
  width: "3px",
  height: "17px",
  backgroundColor: "#0a64ff",
  verticalAlign: "middle",
  marginInlineStart: "7px",
  borderRadius: "2px",
});

export const traffic = cva({
  base: {
    width: "11px",
    height: "11px",
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

export const searchBar = css({
  width: "100%",
  backgroundColor: "white",
  borderRadius: "full",
  boxShadow: "0 3px 12px rgba(60, 60, 140, 0.12)",
  height: "48px",
  display: "flex",
  alignItems: "center",
  paddingInline: "18px",
  gap: "11px",
  fontWeight: "600",
  fontSize: "15px",
  color: "#0d1330",
  boxSizing: "border-box",
  _dark: {
    backgroundColor: "#222222",
    color: "#e8eaf4",
    boxShadow: "0 3px 12px rgba(0, 0, 0, 0.35)",
  },
});

export const googleMark = css({
  fontWeight: "800",
  fontSize: "19px",
  backgroundImage: "conic-gradient(#ea4335 0 25%, #fbbc05 0 50%, #34a853 0 75%, #4285f4 0)",
  backgroundClip: "text",
  color: "transparent",
});

export const searchMag = css({
  marginInlineStart: "auto",
  color: "#666",
  fontSize: "14px",
  display: "inline-flex",
});

export const pill = cva({
  base: {
    position: "absolute",
    left: "50%",
    bottom: "-16px",
    transform: "translateX(-50%)",
    display: "inline-flex",
    alignItems: "center",
    gap: "9px",
    backgroundColor: "background.sub",
    borderRadius: "full",
    paddingTop: "6px",
    paddingBottom: "6px",
    paddingLeft: "7px",
    paddingRight: "18px",
    fontWeight: "700",
    fontSize: "12.5px",
    whiteSpace: "nowrap",
  },
  variants: {
    tone: {
      japanese: {
        color: "#1c3aa8",
        boxShadow: "0 0 0 3px rgba(80, 170, 255, 0.35), 0 6px 20px rgba(50, 150, 255, 0.25)",
        _dark: { color: "#8eb6ff" },
      },
      english: {
        color: "#3a2aa8",
        boxShadow: "0 0 0 3px rgba(160, 120, 255, 0.3), 0 6px 20px rgba(140, 100, 255, 0.22)",
        _dark: { color: "#c4b5fd" },
      },
    },
  },
});

export const pillGlyph = cva({
  base: {
    width: "30px",
    height: "30px",
    borderRadius: "full",
    display: "grid",
    placeItems: "center",
    color: "white",
    fontSize: "17px",
    fontWeight: "700",
    lineHeight: "1",
    flexShrink: "0",
    textDecoration: "none",
  },
  variants: {
    tone: {
      japanese: {
        backgroundImage: "linear-gradient(#3aa0ff, #1a6dff)",
      },
      english: {
        backgroundImage: "linear-gradient(#b17aff, #7f4cf0)",
      },
    },
  },
});

export const hub = css({
  justifySelf: "center",
  width: "108px",
  height: "108px",
  borderRadius: "full",
  backgroundColor: "background.sub",
  boxShadow: "0 8px 28px rgba(50, 70, 140, 0.14)",
  display: "grid",
  placeItems: "center",
  position: "relative",
  _dark: {
    backgroundColor: "#323232",
    boxShadow: "0 8px 28px rgba(0, 0, 0, 0.45)",
  },
});

export const hubLogo = css({
  width: "44px",
  height: "auto",
  objectFit: "contain",
  display: "block",
});

export const hubCycle = css({
  position: "absolute",
  left: "-16px",
  top: "-16px",
  width: "140px",
  height: "140px",
  pointerEvents: "none",
  overflow: "visible",
});

export const features = css({
  width: "100%",
  marginTop: "40px",
  display: "grid",
  gridTemplateColumns: "repeat(3, 1fr)",
});

export const feature = cva({
  base: {
    display: "flex",
    alignItems: "center",
    gap: "12px",
    paddingInline: "16px",
    fontSize: "14px",
    fontWeight: "600",
    lineHeight: "1.5",
    color: "#2a2f4d",
    textAlign: "start",
    whiteSpace: "pre-line",
    _dark: {
      color: "#c8cde0",
    },
  },
  variants: {
    divided: {
      true: {
        borderInlineStartWidth: "1px",
        borderInlineStartStyle: "solid",
        borderInlineStartColor: "#e3e5f1",
        _dark: {
          borderInlineStartColor: "#2a3050",
        },
      },
      false: {},
    },
  },
  defaultVariants: {
    divided: false,
  },
});

export const featureIcon = cva({
  base: {
    flexShrink: "0",
    width: "52px",
    height: "52px",
    borderRadius: "full",
    display: "grid",
    placeItems: "center",
    lineHeight: "1",
    fontSize: "21px",
    "& > svg": {
      display: "block",
      width: "21px",
      height: "21px",
    },
  },
  variants: {
    tone: {
      auto: {
        backgroundColor: "status.info.main",
        color: "white",
      },
      apps: {
        backgroundColor: "#6a3cff",
        color: "white",
      },
      setup: {
        backgroundColor: "#6a6f8a",
        color: "white",
      },
    },
  },
});

export const shortcutTip = css({
  width: "100%",
  maxWidth: "36rem",
  marginTop: "36px",
  display: "flex",
  alignItems: "center",
  gap: "18px",
  paddingBlock: "14px",
  paddingInline: "18px",
  backgroundColor: "background.sub",
  borderRadius: "18px",
  boxShadow: "0 8px 28px rgba(70, 90, 200, 0.12)",
  textAlign: "start",
  boxSizing: "border-box",
  _dark: {
    boxShadow: "0 8px 28px rgba(0, 0, 0, 0.35)",
  },
});

export const shortcutTipKeys = css({
  display: "flex",
  alignItems: "center",
  gap: "8px",
  flexShrink: "0",
});

export const shortcutTipKeyCap = css({
  width: "40px",
  height: "40px",
  borderRadius: "10px",
  backgroundColor: "background.sub",
  display: "grid",
  placeItems: "center",
  fontSize: "18px",
  fontWeight: "600",
  color: "#1c2340",
  boxShadow: "0 4px 12px rgba(70, 100, 220, 0.16), inset 0 0 0 1px #e8ecf8",
  _dark: {
    color: "#e8eaf4",
    boxShadow: "0 4px 12px rgba(0, 0, 0, 0.3), inset 0 0 0 1px #2a3050",
  },
});

export const shortcutTipDivider = css({
  width: "1px",
  alignSelf: "stretch",
  backgroundColor: "#e3e5f1",
  flexShrink: "0",
  _dark: {
    backgroundColor: "#2a3050",
  },
});

export const shortcutTipCopy = css({
  display: "flex",
  flexDirection: "column",
  gap: "4px",
  minWidth: "0",
  flex: "1",
});

export const shortcutTipTitle = css({
  fontSize: "14px",
  fontWeight: "700",
  lineHeight: "1.45",
  color: "#1c2340",
  _dark: {
    color: "#e8eaf4",
  },
});

export const shortcutTipTitleAccent = css({
  color: "#1a6dff",
  _dark: {
    color: "#8eb6ff",
  },
});

export const shortcutTipBody = css({
  fontSize: "12px",
  fontWeight: "500",
  lineHeight: "1.5",
  color: "#6b7190",
  _dark: {
    color: "#a8aec8",
  },
});
