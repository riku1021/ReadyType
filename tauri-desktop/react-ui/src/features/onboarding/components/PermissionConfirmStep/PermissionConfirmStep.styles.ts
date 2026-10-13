import { css, cva } from "@/styled-system/css";

/** HTML モック（ReadyType アクセシビリティが許可されました.html / 800×600） */

export const root = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  width: "100%",
});

export const badge = css({
  position: "relative",
  marginTop: "34px",
  width: "260px",
  height: "116px",
  display: "grid",
  placeItems: "center",
  flexShrink: "0",
});

export const checkAnimationWrap = css({
  width: "260px",
  height: "116px",
  display: "grid",
  placeItems: "center",
});

export const pendingBadge = css({
  width: "108px",
  height: "108px",
  borderRadius: "full",
  backgroundColor: "#eef0fb",
  display: "grid",
  placeItems: "center",
  color: "#6b7190",
  fontSize: "40px",
  boxShadow: "0 8px 28px rgba(70, 90, 200, 0.12)",
  _dark: {
    backgroundColor: "#252a42",
    color: "#a8aec8",
  },
});

export const title = css({
  marginTop: "22px",
  fontSize: "36px",
  fontWeight: "800",
  textAlign: "center",
  letterSpacing: "-0.01em",
  color: "#0d1330",
  lineHeight: "1.25",
  _dark: {
    color: "#e8eaf4",
  },
});

export const titlePending = css({
  marginTop: "22px",
  fontSize: "28px",
  fontWeight: "800",
  textAlign: "center",
  color: "#0d1330",
  lineHeight: "1.35",
  maxWidth: "28rem",
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
  marginTop: "10px",
  textAlign: "center",
  color: "#6b7190",
  fontSize: "15px",
  lineHeight: "1.75",
  fontWeight: "500",
  maxWidth: "36rem",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const leadPending = css({
  marginTop: "10px",
  textAlign: "center",
  color: "#6b7190",
  fontSize: "14px",
  lineHeight: "1.6",
  fontWeight: "500",
  maxWidth: "28rem",
  whiteSpace: "pre-line",
  _dark: {
    color: "#a8aec8",
  },
});

export const statusCard = css({
  marginTop: "30px",
  width: "100%",
  maxWidth: "520px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  gap: "34px",
  paddingBlock: "22px",
  paddingInline: "30px",
  backgroundColor: "background.sub",
  borderRadius: "20px",
  boxShadow: "inset 0 0 0 1px {colors.border.primary}, 0 2px 8px {colors.shadow.primary}",
});

export const fieldIcon = css({
  position: "relative",
  width: "84px",
  height: "64px",
  flexShrink: "0",
});

export const fieldIconBack = css({
  position: "absolute",
  left: "6px",
  top: "4px",
  width: "62px",
  height: "20px",
  borderTopLeftRadius: "12px",
  borderTopRightRadius: "12px",
  backgroundColor: "#cfe2ff",
});

export const fieldIconFront = css({
  position: "absolute",
  left: "6px",
  top: "14px",
  width: "62px",
  height: "48px",
  borderRadius: "12px",
  backgroundColor: "white",
  borderWidth: "2.5px",
  borderStyle: "solid",
  borderColor: "#9cc7ff",
  boxShadow: "0 0 14px rgba(100, 170, 255, 0.45)",
  _after: {
    content: '""',
    position: "absolute",
    left: "8px",
    top: "13px",
    width: "3px",
    height: "20px",
    backgroundColor: "#0a64ff",
    borderRadius: "2px",
  },
  _dark: {
    backgroundColor: "#12162a",
  },
});

export const fieldSpark = cva({
  base: {
    position: "absolute",
    width: "4px",
    height: "12px",
    borderRadius: "3px",
    backgroundColor: "#16c46c",
  },
  variants: {
    index: {
      "0": { left: "66px", top: "0" },
      "1": { left: "76px", top: "8px", transform: "rotate(45deg)" },
      "2": {
        left: "76px",
        top: "24px",
        width: "12px",
        height: "4px",
        transform: "none",
      },
    },
  },
});

export const statusSep = css({
  alignSelf: "stretch",
  width: "1px",
  backgroundColor: "#dfe6ee",
  marginBlock: "2px",
  flexShrink: "0",
  _dark: {
    backgroundColor: "#2a3050",
  },
});

export const okRow = css({
  display: "flex",
  alignItems: "center",
  gap: "14px",
  textAlign: "start",
});

export const okCheck = cva({
  base: {
    flexShrink: "0",
    width: "36px",
    height: "36px",
    borderRadius: "full",
    display: "grid",
    placeItems: "center",
    color: "white",
    fontSize: "18px",
  },
  variants: {
    tone: {
      granted: {
        backgroundColor: "#16c46c",
        _after: {
          content: '""',
          width: "8px",
          height: "15px",
          borderStyle: "solid",
          borderColor: "white",
          borderWidth: "0 3.5px 3.5px 0",
          transform: "translateY(-1px) rotate(45deg)",
        },
      },
      pending: {
        backgroundColor: "#6b7190",
      },
    },
  },
  defaultVariants: {
    tone: "granted",
  },
});

export const okTitle = css({
  display: "block",
  fontSize: "19px",
  fontWeight: "800",
  color: "#0d1330",
  _dark: {
    color: "#e8eaf4",
  },
});

export const okSubtitle = css({
  display: "block",
  marginTop: "6px",
  color: "#6b7190",
  fontSize: "14px",
  fontWeight: "500",
  _dark: {
    color: "#a8aec8",
  },
});
