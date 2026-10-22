import { css, cva } from "styled-system/css";

export const marquee = css({
  bg: "ink",
  py: "20px",
  display: "grid",
  gap: "14px",
  overflow: "clip",
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

export const track = css({
  display: "flex",
  alignItems: "center",
  gap: "16px",
  width: "max-content",
  animation: "mqa 42s linear infinite",
});

export const chip = css({
  display: "inline-flex",
  alignItems: "center",
  gap: "12px",
  color: "white",
  fontWeight: "black",
  fontSize: "clamp(0.875rem, 1.6vw, 1rem)",
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
});

export const chipMark = cva({
  base: {
    width: "36px",
    height: "36px",
    borderRadius: "50%",
    display: "grid",
    placeItems: "center",
    fontWeight: "black",
    fontSize: "1rem",
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
