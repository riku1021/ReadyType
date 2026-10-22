import { css, cva } from "styled-system/css";

export const hero = css({
  position: "relative",
  overflow: "clip",
  pt: "clamp(40px, 7vw, 72px)",
  pb: "clamp(64px, 9vw, 110px)",
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
  display: "flex",
  flexWrap: "wrap",
  gap: "clamp(32px, 5vw, 48px)",
  alignItems: "center",
  "& > :first-child": { flex: "1.05 1 min(100%, 420px)" },
});

export const title = css({
  fontSize: "clamp(2.375rem, 5.8vw, 4rem)",
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
  fontSize: "1.125rem",
  color: "muted",
  mb: "32px",
  maxWidth: "30rem",
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
  fontSize: "0.8125rem",
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

export const stageBox = css({
  position: "relative",
  flex: "1 1 min(100%, 420px)",
  maxWidth: "600px",
  mx: "auto",
  containerType: "inline-size",
  "--bl": "clamp(0px, 2.4vw, 30px)",
});

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
    fontSize: "0.8125rem",
    display: "flex",
    gap: "8px",
    alignItems: "center",
    zIndex: "float",
    animation: "bob 5s ease-in-out infinite",
  },
  variants: {
    place: {
      auto: { left: "calc(var(--bl) * -1)", top: "36%", _stageTight: { display: "none" } },
      rules: { right: "-8px", bottom: "-14px", animationDelay: "-2s" },
      privacy: {
        left: "6%",
        top: "-22px",
        animationDelay: "-3.2s",
        _stageTight: { display: "none" },
      },
      shortcut: { right: "calc(var(--bl) * -1)", top: "30%", animationDelay: "-1s" },
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
  fontSize: "0.75rem",
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
  fontSize: "0.75rem",
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
    fontSize: "1.125rem",
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
        fontSize: "0.9375rem",
        animation: "r2 6s infinite",
      },
    },
  },
});

export const chip = cva({
  base: {
    position: "absolute",
    right: "calc(var(--bl) * -0.4)",
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

export const chipMark = css({ fontSize: "1.75rem", display: "block" });
export const chipLabel = css({ fontSize: "0.6562rem", opacity: "0.9", fontWeight: "black" });
