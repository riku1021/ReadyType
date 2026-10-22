import { css, cva } from "styled-system/css";

const headingBase = {
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  fontSize: "clamp(1.75rem, 4.2vw, 2.625rem)",
  mb: "14px",
} as const;

export const heading = css({
  ...headingBase,
  textAlign: "center",
});

export const lead = css({
  textAlign: "center",
  color: "muted",
  maxWidth: "640px",
  mx: "auto",
  mb: "clamp(32px, 5vw, 56px)",
});

export const eyebrow = cva({
  base: {
    display: "table",
    fontFamily: "latin",
    fontWeight: "extrabold",
    fontSize: "0.8125rem",
    lineHeight: "1",
    letterSpacing: "0.14em",
    px: "16px",
    py: "8px",
    borderRadius: "pill",
  },
  variants: {
    tone: {
      brand: {
        mx: "auto",
        mb: "14px",
        color: "eyebrowInk",
        bg: "soft",
        boxShadow: "inset 0 0 0 1px var(--colors-eyebrow-ring)",
      },
      start: {
        mb: "14px",
        color: "eyebrowInk",
        bg: "soft",
        boxShadow: "inset 0 0 0 1px var(--colors-eyebrow-ring)",
      },
      onDark: {
        display: "inline-block",
        mb: "14px",
        color: "mistBlue",
        bg: "rgba(78, 141, 255, 0.16)",
        boxShadow: "none",
        _howTight: { display: "none" },
      },
    },
  },
  defaultVariants: { tone: "brand" },
});

export const gradText = css({
  textGradient: "brand",
});

export const heroGrad = css({
  textGradient: "shine",
  backgroundSize: "200% 100%",
  animation: "shine 6s linear infinite",
});

export const howGrad = css({
  textGradient: "howText",
});
