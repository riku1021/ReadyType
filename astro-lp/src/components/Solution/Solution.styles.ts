import { css, cva } from "styled-system/css";

export const solution = css({
  bgGradient: "solution",
  borderTopRadius: "clamp(32px, 5vw, 56px)",
  mt: "-1px",
});

const conic =
  "conic-gradient(from var(--ang), var(--colors-brand), var(--colors-accent), var(--colors-gold), var(--colors-brand)) border-box";

export const glanceWrap = css({
  containerType: "inline-size",
  maxWidth: "1000px",
  mx: "auto",
  mb: "clamp(24px, 4vw, 34px)",
});

export const glance = css({
  display: "grid",
  gridTemplateColumns: "minmax(0, 1fr) auto minmax(0, 1fr)",
  gap: "clamp(12px, 2vw, 18px)",
  alignItems: "stretch",
  maxWidth: "none",
  m: "0",
  _glanceTight: { gridTemplateColumns: "minmax(0, 1fr)" },
});

export const glanceCard = cva({
  base: {
    borderRadius: "30px",
    p: "clamp(18px, 3vw, 26px) clamp(16px, 2.6vw, 24px)",
    borderWidth: "2px",
    borderStyle: "solid",
    borderColor: "line",
    borderBottomWidth: "7px",
    bg: "white",
    display: "grid",
    gap: "16px",
    alignContent: "start",
  },
  variants: {
    tone: {
      bad: {},
      good: {
        borderWidth: "3px",
        borderColor: "transparent",
        boxShadow: "good",
        background: `linear-gradient(180deg, var(--colors-white), var(--colors-good-wash)) padding-box, ${conic}`,
        animation: "angr 7s linear infinite",
      },
    },
  },
});

export const glanceTag = css({ fontWeight: "black", fontSize: "1.0625rem" });

export const glanceSurface = cva({
  base: { borderRadius: "20px", p: "16px" },
  variants: {
    tone: {
      bad: { bg: "canvas" },
      good: { bg: "white" },
    },
  },
});

export const glanceLabel = css({
  fontSize: "0.75rem",
  fontWeight: "extrabold",
  color: "muted",
  mb: "8px",
});

export const glanceField = cva({
  base: {
    position: "relative",
    bg: "white",
    borderWidth: "2px",
    borderStyle: "solid",
    borderColor: "line",
    borderRadius: "14px",
    px: "16px",
    py: "14px",
    fontWeight: "extrabold",
    fontSize: "1.125rem",
    display: "flex",
    flexWrap: "wrap",
    alignItems: "center",
    gap: "6px",
    minHeight: "56px",
  },
  variants: {
    surface: { chat: {}, code: {} },
    tone: { bad: {}, good: {} },
  },
  compoundVariants: [
    {
      surface: "chat",
      tone: "bad",
      css: { borderColor: "dangerLine", bg: "dangerWash" },
    },
    {
      surface: "code",
      tone: "bad",
      css: {
        bg: "dangerPink",
        borderColor: "dangerPinkLine",
        color: "dangerText",
        fontFamily: "mono",
        fontWeight: "semibold",
        fontSize: "0.9375rem",
      },
    },
    {
      surface: "code",
      tone: "good",
      css: {
        bgGradient: "codeField",
        borderColor: "codeLine",
        color: "code",
        fontFamily: "mono",
        fontWeight: "semibold",
        fontSize: "0.9375rem",
      },
    },
  ],
});

export const cross = css({
  ml: "auto",
  color: "danger",
  fontWeight: "black",
  fontSize: "1.25rem",
  fontStyle: "normal",
  animation: "shk 2.6s infinite",
});

export const lang = cva({
  base: {
    ml: "auto",
    color: "white",
    fontWeight: "black",
    fontSize: "0.7812rem",
    px: "12px",
    py: "5px",
    borderRadius: "pill",
    whiteSpace: "nowrap",
    fontStyle: "normal",
  },
  variants: {
    tone: {
      ja: { bg: "brand" },
      en: { bg: "accent" },
    },
  },
});

export const caption = cva({
  base: { fontSize: "0.8438rem", fontWeight: "extrabold", mt: "8px" },
  variants: {
    tone: {
      bad: { color: "danger" },
      good: { color: "brand" },
    },
  },
});

export const mid = css({
  alignSelf: "center",
  width: "54px",
  height: "54px",
  borderRadius: "50%",
  bg: "white",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "line",
  display: "grid",
  placeItems: "center",
  fontWeight: "black",
  fontSize: "1.5rem",
  color: "brand",
  boxShadow: "card",
  animation: "nudge 1.4s ease-in-out infinite",
  _glanceTight: { transform: "rotate(90deg)", justifySelf: "center" },
});

export const points = css({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 230px), 1fr))",
  gap: "clamp(14px, 2vw, 20px)",
  maxWidth: "980px",
  mx: "auto",
  mb: "36px",
});

export const point = css({
  bg: "white",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "line",
  borderBottomWidth: "6px",
  borderRadius: "card",
  p: "24px 22px",
  textAlign: "center",
  transition: ".25s",
  _hover: { transform: "translateY(-6px)", borderColor: "hoverLine" },
});

export const pointIcon = css({
  display: "grid",
  placeItems: "center",
  width: "56px",
  height: "56px",
  borderRadius: "18px",
  bg: "soft",
  fontSize: "1.75rem",
  mx: "auto",
  mb: "10px",
  fontStyle: "normal",
});

export const pointTitle = css({
  fontSize: "1.0625rem",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "4px",
});

export const pointCopy = css({ fontSize: "0.875rem", color: "muted" });

export const boost = css({
  maxWidth: "980px",
  mx: "auto",
  borderRadius: "30px",
  color: "white",
  p: "clamp(20px, 3vw, 26px) clamp(20px, 3.4vw, 32px)",
  display: "flex",
  gap: "20px",
  alignItems: "center",
  flexWrap: "wrap",
  position: "relative",
  overflow: "hidden",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "transparent",
  background: `linear-gradient(var(--colors-ink), var(--colors-ink)) padding-box, ${conic}`,
  animation: "angr 7s linear infinite",
  "&::after": {
    content: '""',
    position: "absolute",
    right: "-50px",
    top: "-70px",
    width: "240px",
    height: "240px",
    borderRadius: "50%",
    bgGradient: "brand",
    opacity: "0.5",
    filter: "blur(50px)",
  },
  "& > *": { position: "relative", zIndex: "1" },
});

export const boostIcon = css({ fontSize: "2.375rem", animation: "bob 3s ease-in-out infinite" });

export const boostCopy = css({ flex: "1", minWidth: "240px" });

export const boostTitle = css({
  fontSize: "clamp(1.25rem, 2.6vw, 1.625rem)",
  display: "block",
  fontWeight: "black",
  lineHeight: "1.35",
});

export const boostSub = css({ color: "mist", fontSize: "0.875rem" });
