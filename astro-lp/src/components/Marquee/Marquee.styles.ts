import { css, cva } from "styled-system/css";

export const marquee = css({
  bg: "ink",
  py: "20px",
  display: "grid",
  gap: "14px",
  overflow: "hidden",
  position: "relative",
  "&::before": {
    content: '""',
    position: "absolute",
    inset: "0",
    pointerEvents: "none",
    background:
      "radial-gradient(40% 120% at 15% 50%, rgba(26, 107, 255, 0.35), transparent 70%), radial-gradient(40% 120% at 85% 50%, rgba(124, 92, 255, 0.35), transparent 70%)",
  },
});

export const row = css({
  overflow: "hidden",
  position: "relative",
  maskImage: "linear-gradient(90deg, transparent, #000 6%, #000 94%, transparent)",
  WebkitMaskImage: "linear-gradient(90deg, transparent, #000 6%, #000 94%, transparent)",
});

export const track = cva({
  base: {
    display: "flex",
    alignItems: "center",
    gap: "16px",
    width: "max-content",
  },
  variants: {
    direction: {
      forward: { animation: "mqa 42s linear infinite" },
      reverse: { animation: "mqa 56s linear infinite reverse" },
    },
  },
});

export const chip = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "12px",
  color: "white",
  fontWeight: "black",
  fontSize: "16px",
  whiteSpace: "nowrap",
  py: "7px",
  pr: "22px",
  pl: "7px",
  borderRadius: "pill",
  bg: "rgba(255, 255, 255, 0.07)",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "rgba(255, 255, 255, 0.18)",
  backdropFilter: "blur(6px)",
  _bp700: { fontSize: "14px" },
});

export const chipMark = cva({
  base: {
    width: "36px",
    height: "36px",
    borderRadius: "50%",
    display: "grid",
    placeItems: "center",
    fontWeight: "black",
    fontSize: "16px",
    color: "white",
    fontStyle: "normal",
  },
  variants: {
    tone: {
      brand: { bgGradient: "brandSoft" },
      accent: { bgGradient: "accentSoft" },
      gold: { bgGradient: "goldSoft" },
      success: { bgGradient: "success" },
    },
  },
});

export const outline = css({
  fontWeight: "black",
  fontSize: "40px",
  lineHeight: "1.2",
  whiteSpace: "nowrap",
  color: "transparent",
  WebkitTextStroke: "1.4px rgba(255, 255, 255, 0.6)",
  letterSpacing: "0.02em",
  _bp700: { fontSize: "28px" },
});

export const outlineFill = css({
  fontWeight: "black",
  fontSize: "40px",
  lineHeight: "1.2",
  whiteSpace: "nowrap",
  fontFamily: "latin",
  letterSpacing: "0.12em",
  WebkitTextStroke: "0",
  textGradient: "outlineText",
  _bp700: { fontSize: "28px" },
});

export const spark = css({
  color: "gold",
  fontSize: "26px",
  animation: "orbit 8s linear infinite",
  display: "inline-block",
});
