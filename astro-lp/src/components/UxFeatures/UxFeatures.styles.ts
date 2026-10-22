import { css, cva } from "styled-system/css";

export const grid = css({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 270px), 1fr))",
  gap: "clamp(16px, 2.4vw, 22px)",
});

export const card = css({
  bg: "white",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "line",
  borderBottomWidth: "6px",
  borderRadius: "card",
  p: "clamp(22px, 3vw, 30px) clamp(20px, 2.6vw, 26px)",
  boxShadow: "card",
  transition: ".2s",
  overflow: "hidden",
  _hover: { transform: "translateY(-6px)", borderColor: "hoverLine" },
  "&::after": {
    content: '""',
    position: "absolute",
    right: "-40px",
    top: "-40px",
    width: "120px",
    height: "120px",
    borderRadius: "50%",
    bgGradient: "brand",
    opacity: "0.08",
    transition: ".4s",
    pointerEvents: "none",
  },
  "&:hover::after": { transform: "scale(2.4)", opacity: "0.12" },
  "&:hover [data-icon]": { animation: "wig .6s" },
});

export const visual = css({
  height: "72px",
  display: "flex",
  alignItems: "center",
  gap: "8px",
  mb: "12px",
  fontWeight: "black",
  fontSize: "0.875rem",
});

export const visualStack = css({
  height: "auto",
  flexDirection: "column",
  alignItems: "stretch",
  gap: "6px",
  justifyContent: "center",
  mb: "14px",
});

export const search = css({
  bg: "white",
  borderRadius: "12px",
  px: "10px",
  py: "6px",
  fontSize: "0.75rem",
  fontWeight: "bold",
  color: "muted",
});

export const appRow = css({
  display: "flex",
  alignItems: "center",
  gap: "8px",
  bg: "white",
  borderRadius: "12px",
  px: "10px",
  py: "6px",
  fontWeight: "extrabold",
  fontSize: "0.7812rem",
  transition: "transform .35s",
  "[data-feature]:hover &": {
    "&[data-shift='true']": { transform: "translateX(6px)" },
  },
});

export const miniTag = cva({
  base: {
    fontStyle: "normal",
    fontSize: "0.6562rem",
    fontWeight: "black",
    color: "white",
    px: "8px",
    py: "1px",
    borderRadius: "pill",
  },
  variants: {
    tone: {
      ja: { bg: "brand" },
      en: { bg: "accent" },
    },
    edge: {
      true: { ml: "auto" },
      false: {},
    },
  },
});

export const toggle = css({
  ml: "6px",
  width: "30px",
  height: "18px",
  borderRadius: "pill",
  bg: "brand",
  position: "relative",
  "&::after": {
    content: '""',
    position: "absolute",
    right: "3px",
    top: "3px",
    width: "12px",
    height: "12px",
    borderRadius: "50%",
    bg: "white",
  },
});

export const dots = css({ display: "flex", gap: "6px", justifyContent: "center" });

export const dot = css({
  width: "8px",
  height: "8px",
  borderRadius: "50%",
  bg: "dot",
  "&[data-on='true']": { width: "24px", borderRadius: "8px", bg: "brand" },
});

export const welcome = css({
  bg: "white",
  borderRadius: "12px",
  p: "8px",
  textAlign: "center",
  fontWeight: "black",
  fontSize: "0.8125rem",
});

export const cardTitle = css({
  fontSize: "1.125rem",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "6px",
});

export const cardCopy = css({ fontSize: "0.9062rem", color: "muted" });

export const privacy = css({
  position: "relative",
  overflow: "hidden",
  bg: "ink",
  color: "white",
  borderRadius: "36px",
  display: "flex",
  flexWrap: "wrap",
  gap: "28px",
  alignItems: "center",
  mt: "28px",
  p: "clamp(26px, 4vw, 44px) clamp(22px, 4vw, 40px)",
  "&::after": {
    content: '""',
    position: "absolute",
    right: "-60px",
    top: "-60px",
    width: "220px",
    height: "220px",
    borderRadius: "50%",
    bgGradient: "brand",
    opacity: "0.35",
    filter: "blur(40px)",
  },
  "& > *": { position: "relative", zIndex: "1" },
});

export const privacyBody = css({ flex: "1 1 260px" });

export const privacyIcon = css({
  flex: "none",
  width: "72px",
  height: "72px",
  borderRadius: "50%",
  bgGradient: "brand",
  display: "grid",
  placeItems: "center",
  fontSize: "1.875rem",
});

export const privacyTitle = css({
  fontSize: "1.375rem",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "6px",
});

export const privacyCopy = css({ color: "mist", fontSize: "0.9375rem" });
