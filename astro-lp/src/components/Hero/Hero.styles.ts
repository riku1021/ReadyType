import { css, cva } from "styled-system/css";

export const hero = css({
  position: "relative",
  overflow: "hidden",
  pt: "72px",
  pb: "110px",
  "&::before": {
    content: '""',
    position: "absolute",
    inset: "0",
    background:
      "radial-gradient(520px circle at var(--mx, 72%) var(--my, 28%), rgba(124, 92, 255, 0.18), transparent 60%)",
    zIndex: "-1",
    pointerEvents: "none",
  },
  "&::after": {
    content: '""',
    position: "absolute",
    inset: "0",
    backgroundImage: "radial-gradient(var(--colors-eyebrow-ring) 1.3px, transparent 1.5px)",
    backgroundSize: "28px 28px",
    maskImage: "linear-gradient(180deg, #000, transparent 88%)",
    WebkitMaskImage: "linear-gradient(180deg, #000, transparent 88%)",
    zIndex: "-1",
    opacity: "0.55",
    pointerEvents: "none",
  },
});

export const aurora = css({
  position: "absolute",
  inset: "-10% -10% auto -10%",
  height: "95%",
  zIndex: "-1",
  pointerEvents: "none",
  filter: "blur(36px)",
  background:
    "radial-gradient(40% 50% at 18% 30%, rgba(26, 107, 255, 0.3), transparent 70%), radial-gradient(35% 45% at 78% 22%, rgba(124, 92, 255, 0.32), transparent 70%), radial-gradient(30% 40% at 55% 78%, rgba(255, 176, 120, 0.24), transparent 70%)",
  animation: "aur 16s ease-in-out infinite alternate",
});

export const orb = cva({
  base: {
    position: "absolute",
    borderRadius: "50%",
    filter: "blur(70px)",
    zIndex: "-1",
  },
  variants: {
    place: {
      right: {
        width: "480px",
        height: "480px",
        bg: "orbBlue",
        right: "-120px",
        top: "-60px",
        transform: "translateY(calc(var(--sy, 0) * -0.14px))",
      },
      left: {
        width: "380px",
        height: "380px",
        bg: "orbLilac",
        left: "-120px",
        bottom: "-120px",
        transform: "translateY(calc(var(--sy, 0) * -0.08px))",
      },
    },
  },
});

export const grid = css({
  display: "grid",
  gridTemplateColumns: "1.05fr 1fr",
  gap: "48px",
  alignItems: "center",
  _bp900: { gridTemplateColumns: "1fr" },
});

export const title = css({
  fontSize: "clamp(38px, 5.8vw, 64px)",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "22px",
});

const wordBase = {
  display: "inline-block",
  opacity: "0",
  transform: "translateY(0.55em)",
  animation: "wd .9s cubic-bezier(.2,.7,.2,1) forwards",
} as const;

export const word = [
  css({ ...wordBase, animationDelay: ".05s" }),
  css({ ...wordBase, animationDelay: ".18s" }),
  css({ ...wordBase, animationDelay: ".34s" }),
  css({ ...wordBase, animationDelay: ".5s" }),
];

export const copy = css({
  fontSize: "18px",
  color: "muted",
  mb: "32px",
  maxWidth: "480px",
  animation: "up 1s .5s both cubic-bezier(.2,.7,.2,1)",
});

export const actions = css({
  display: "flex",
  gap: "14px",
  flexWrap: "wrap",
  animation: "up 1s .5s both cubic-bezier(.2,.7,.2,1)",
});

export const pills = css({
  display: "flex",
  gap: "10px",
  flexWrap: "wrap",
  mt: "28px",
  fontSize: "13px",
  fontWeight: "extrabold",
  color: "muted",
  animation: "up 1s .5s both cubic-bezier(.2,.7,.2,1)",
});

export const pill = css({
  bg: "white",
  borderRadius: "pill",
  px: "14px",
  py: "6px",
  boxShadow: "inset 0 0 0 1px var(--colors-line)",
});

export const stageBox = css({ position: "relative" });

export const wave = css({
  position: "absolute",
  inset: "-30px",
  borderRadius: "50px",
  background: "conic-gradient(from 0deg, #1A6BFF22, #7C5CFF33, #1A6BFF22)",
  filter: "blur(30px)",
  zIndex: "-1",
  animation: "orbit 14s linear infinite",
});

export const floatCard = cva({
  base: {
    position: "absolute",
    bg: "white",
    borderRadius: "16px",
    boxShadow: "card",
    borderWidth: "1px",
    borderStyle: "solid",
    borderColor: "line",
    px: "14px",
    py: "10px",
    fontWeight: "black",
    fontSize: "13px",
    display: "flex",
    gap: "8px",
    alignItems: "center",
    zIndex: "float",
    animation: "bob 5s ease-in-out infinite",
    _bp900: { display: "none" },
  },
  variants: {
    place: {
      auto: { left: "-30px", top: "36%" },
      rules: { right: "-8px", bottom: "-14px", animationDelay: "-2s" },
      privacy: { left: "6%", top: "-22px", animationDelay: "-3.2s" },
      shortcut: { right: "-30px", top: "30%", animationDelay: "-1s" },
    },
  },
});

export const floatIcon = css({
  width: "26px",
  height: "26px",
  borderRadius: "50%",
  bgGradient: "brand",
  display: "grid",
  placeItems: "center",
  color: "white",
  fontSize: "12px",
});

export const stage = css({
  display: "grid",
  gap: "26px",
  py: "10px",
  px: "16px",
  perspective: "1200px",
  transition: "transform .25s ease-out",
  transformStyle: "preserve-3d",
});

export const mock = cva({
  base: { pb: "20px" },
  variants: {
    scene: {
      ja: { animation: "f1 6s infinite" },
      en: { animation: "f2 6s infinite" },
    },
  },
});

export const mockBody = css({ pt: "10px", px: "20px" });

export const mockLabel = css({
  fontSize: "12px",
  fontWeight: "extrabold",
  color: "muted",
  mb: "6px",
});

export const field = cva({
  base: {
    borderWidth: "2px",
    borderStyle: "solid",
    borderColor: "line",
    borderRadius: "14px",
    px: "16px",
    py: "12px",
    fontSize: "18px",
    fontWeight: "extrabold",
    minHeight: "54px",
    display: "flex",
    alignItems: "center",
    bg: "white",
  },
  variants: {
    scene: {
      ja: { animation: "r1 6s infinite" },
      en: {
        bgGradient: "codeField",
        color: "code",
        borderColor: "codeLine",
        fontFamily: "mono",
        fontWeight: "semibold",
        fontSize: "15px",
        animation: "r2 6s infinite",
      },
    },
  },
});

export const chip = cva({
  base: {
    position: "absolute",
    right: "-12px",
    top: "-16px",
    width: "70px",
    height: "70px",
    borderRadius: "20px",
    display: "grid",
    placeItems: "center",
    textAlign: "center",
    fontWeight: "black",
    lineHeight: "1.1",
    color: "white",
    boxShadow: "glow",
  },
  variants: {
    tone: {
      ja: { bgGradient: "brandSoft", animation: "p1 6s infinite" },
      en: { bgGradient: "accentSoft", animation: "p2 6s infinite" },
    },
  },
});

export const chipMark = css({ fontSize: "28px", display: "block" });
export const chipLabel = css({ fontSize: "10.5px", opacity: "0.9", fontWeight: "black" });
