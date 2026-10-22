import { css, cva } from "styled-system/css";

export const split = css({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 420px), 1fr))",
  gap: "clamp(32px, 5vw, 56px)",
  alignItems: "center",
});

export const title = css({
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  fontSize: "clamp(1.75rem, 4.2vw, 2.625rem)",
  textAlign: "start",
  mb: "14px",
});

export const copy = css({ color: "muted", mb: "18px" });

export const checks = css({ listStyle: "none" });

export const check = css({
  position: "relative",
  pl: "28px",
  fontWeight: "extrabold",
  fontSize: "0.9062rem",
  mb: "6px",
  "&::before": {
    content: '"✓"',
    position: "absolute",
    left: "0",
    top: "3px",
    width: "20px",
    height: "20px",
    borderRadius: "50%",
    bgGradient: "brand",
    color: "white",
    fontSize: "0.6875rem",
    display: "grid",
    placeItems: "center",
  },
});

export const list = css({
  display: "grid",
  gap: "14px",
  p: "20px 22px 24px",
});

export const rule = css({
  display: "flex",
  flexWrap: "wrap",
  alignItems: "center",
  gap: "12px",
  p: "14px 16px",
  borderRadius: "16px",
  bg: "canvas",
  fontWeight: "extrabold",
  fontSize: "0.875rem",
  transition: ".25s",
  _hover: { transform: "translateX(6px)", bg: "soft" },
});

export const aside = css({ ml: "6px", fontWeight: "medium", color: "muted", fontSize: "0.75rem" });

export const hint = css({
  display: "block",
  fontWeight: "medium",
  color: "muted",
  fontSize: "0.75rem",
});

export const tag = cva({
  base: {
    ml: "auto",
    borderRadius: "pill",
    px: "12px",
    py: "3px",
    fontSize: "0.7188rem",
    fontWeight: "black",
    color: "white",
  },
  variants: {
    tone: {
      ja: { bg: "brand" },
      en: { bg: "accent" },
    },
  },
});
