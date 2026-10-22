import { css, cva } from "styled-system/css";

export const split = css({
  display: "grid",
  gridTemplateColumns: "1fr 1fr",
  gap: "56px",
  alignItems: "center",
  _bp900: { gridTemplateColumns: "1fr" },
});

export const title = css({
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  fontSize: "clamp(28px, 4.2vw, 42px)",
  textAlign: "left",
  mb: "14px",
  _bp900: { textAlign: "center" },
});

export const copy = css({ color: "muted", mb: "18px" });

export const checks = css({ listStyle: "none" });

export const check = css({
  position: "relative",
  pl: "28px",
  fontWeight: "extrabold",
  fontSize: "14.5px",
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
    fontSize: "11px",
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
  alignItems: "center",
  gap: "12px",
  p: "14px 16px",
  borderRadius: "16px",
  bg: "canvas",
  fontWeight: "extrabold",
  fontSize: "14px",
  transition: ".25s",
  _hover: { transform: "translateX(6px)", bg: "soft" },
});

export const aside = css({ ml: "6px", fontWeight: "medium", color: "muted", fontSize: "12px" });

export const hint = css({
  display: "block",
  fontWeight: "medium",
  color: "muted",
  fontSize: "12px",
});

export const tag = cva({
  base: {
    ml: "auto",
    borderRadius: "pill",
    px: "12px",
    py: "3px",
    fontSize: "11.5px",
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
