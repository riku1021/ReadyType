import { css } from "styled-system/css";

export const how = css({
  bg: "howBg",
  color: "white",
  py: "0",
  position: "relative",
});

export const track = css({
  height: "340vh",
  position: "relative",
  _bp900: { height: "300vh" },
});

export const stick = css({
  position: "sticky",
  top: "0",
  height: "100vh",
  display: "flex",
  alignItems: "center",
  overflow: "hidden",
  "&::before": {
    content: '""',
    position: "absolute",
    width: "640px",
    height: "640px",
    right: "-120px",
    top: "-120px",
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(78, 141, 255, 0.35), transparent 65%)",
  },
  "&::after": {
    content: '""',
    position: "absolute",
    width: "520px",
    height: "520px",
    left: "-160px",
    bottom: "-200px",
    borderRadius: "50%",
    background: "radial-gradient(circle, rgba(124, 92, 255, 0.3), transparent 65%)",
  },
});

export const grid = css({
  display: "grid",
  gridTemplateColumns: "0.9fr 1.1fr",
  gap: "48px",
  alignItems: "center",
  width: "100%",
  position: "relative",
  zIndex: "1",
  _bp900: { gridTemplateColumns: "1fr", gap: "6px" },
});

export const title = css({
  color: "white",
  textAlign: "left",
  fontSize: "clamp(28px, 3.6vw, 46px)",
  lineHeight: "1.35",
  fontWeight: "black",
  letterSpacing: "0.01em",
  mb: "14px",
  _bp900: { fontSize: "23px", mb: "8px" },
});

export const lead = css({
  textAlign: "left",
  m: "0 0 30px",
  color: "slate",
  fontSize: "16px",
  _bp900: { display: "none" },
});

export const steps = css({
  listStyle: "none",
  display: "grid",
  gap: "12px",
  p: "0",
});

export const step = css({
  position: "relative",
  overflow: "hidden",
  display: "flex",
  gap: "16px",
  alignItems: "center",
  py: "16px",
  px: "18px",
  borderRadius: "22px",
  bg: "rgba(255, 255, 255, 0.04)",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "rgba(255, 255, 255, 0.1)",
  cursor: "pointer",
  transition: "background .4s, border-color .4s, opacity .4s",
  ".js &": {
    opacity: "0.5",
    "&[data-state='act']": { opacity: "0.78" },
    "&[data-state='cur']": {
      opacity: "1",
      bg: "rgba(78, 141, 255, 0.14)",
      borderColor: "brandBright",
    },
  },
  "&::after": {
    content: '""',
    position: "absolute",
    left: "0",
    bottom: "0",
    height: "3px",
    width: "0",
    bgGradient: "brand",
  },
  "&[data-state='cur']::after": { width: "calc(var(--lp, 0) * 100%)" },
  _bp900: { py: "10px", px: "14px", gap: "12px" },
});

export const ring = css({
  position: "relative",
  zIndex: "1",
  flex: "none",
  width: "54px",
  height: "54px",
  borderRadius: "16px",
  bg: "rgba(255, 255, 255, 0.08)",
  display: "grid",
  placeItems: "center",
  fontSize: "24px",
  overflow: "hidden",
  transition: "background .4s",
  "[data-state='cur'] &": { bgGradient: "brand" },
  ".js [data-state='act'] &": {
    "&::after": {
      content: '"✓"',
      position: "absolute",
      inset: "0",
      bgGradient: "success",
      display: "grid",
      placeItems: "center",
      fontWeight: "black",
      fontSize: "22px",
      color: "white",
    },
  },
  _bp900: { width: "42px", height: "42px", fontSize: "19px", borderRadius: "12px" },
});

export const stepTitle = css({
  fontSize: "22px",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "2px",
  _bp900: { fontSize: "16px" },
});

export const stepText = css({
  color: "stepText",
  fontSize: "15px",
  lineHeight: "1.7",
  ".js &": {
    maxHeight: "0",
    opacity: "0",
    overflow: "hidden",
    transition: "max-height .5s, opacity .4s",
  },
  ".js [data-state='cur'] &": { maxHeight: "5em", opacity: "1" },
  _bp900: { fontSize: "12.5px", lineHeight: "1.5" },
});

export const stage = css({
  perspective: "1700px",
  display: "flex",
  justifyContent: "center",
  py: "30px",
  _bp900: { order: "-1", py: "6px", pb: "26px" },
});

export const laptop = css({
  position: "relative",
  width: "100%",
  maxWidth: "640px",
  transformStyle: "preserve-3d",
  transform: "rotateX(10deg) rotateY(-22deg)",
  transition: "transform 1s cubic-bezier(.2,.7,.2,1)",
  "[data-step='1'] &": { transform: "rotateX(6deg) rotateY(-6deg)" },
  "[data-step='2'] &": { transform: "rotateX(8deg) rotateY(14deg)" },
  _bp900: { maxWidth: "300px" },
});

export const lid = css({
  aspectRatio: "16 / 10",
  bg: "lid",
  borderRadius: "18px",
  p: "2.4%",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "lidLine",
  boxShadow: "0 40px 90px rgba(0, 0, 0, 0.6), 0 0 90px rgba(78, 141, 255, 0.28)",
});

export const display = css({
  position: "relative",
  width: "100%",
  height: "100%",
  borderRadius: "8px",
  overflow: "hidden",
  bgGradient: "screen",
  containerType: "inline-size",
  fontSize: "max(6px, 2cqw)",
  color: "white",
});

export const base = css({
  height: "15px",
  mx: "-5%",
  backgroundImage: "linear-gradient(#D9DFEE, #98A2BE)",
  borderRadius: "2px 2px 18px 18px",
  position: "relative",
  "&::before": {
    content: '""',
    position: "absolute",
    left: "50%",
    top: "0",
    width: "18%",
    height: "5px",
    bg: "#B2BAD2",
    transform: "translateX(-50%)",
    borderRadius: "0 0 8px 8px",
  },
});

export const shadow = css({
  position: "absolute",
  left: "6%",
  right: "6%",
  bottom: "-40px",
  height: "34px",
  background: "radial-gradient(ellipse, rgba(0, 0, 0, 0.6), transparent 70%)",
  filter: "blur(10px)",
});

export const menu = css({
  display: "flex",
  gap: "1.4em",
  alignItems: "center",
  py: "0.5em",
  px: "1.2em",
  bg: "rgba(15, 27, 61, 0.55)",
  fontWeight: "bold",
});

export const menuGrow = css({ flex: "1" });

export const ime = css({
  minWidth: "2.6em",
  textAlign: "center",
  borderRadius: "0.6em",
  px: "0.5em",
  py: "0.1em",
  fontWeight: "black",
  bg: "accent",
  transition: ".4s",
  "[data-step='2'] &": {
    bg: "brand",
    boxShadow: "0 0 0 0.4em rgba(255, 255, 255, 0.35)",
  },
});

export const imeEn = css({ "[data-step='2'] &": { display: "none" } });
export const imeJa = css({ display: "none", "[data-step='2'] &": { display: "inline" } });

export const workspace = css({
  position: "absolute",
  left: "5%",
  right: "5%",
  top: "14%",
  bottom: "8%",
  bg: "screen",
  borderRadius: "1.2em",
  display: "grid",
  gridTemplateColumns: "0.9fr 1.1fr",
  overflow: "hidden",
  boxShadow: "0 1.6em 4em rgba(15, 27, 61, 0.5)",
});

export const column = css({
  py: "1em",
  px: "1.2em",
  display: "flex",
  flexDirection: "column",
  gap: "0.8em",
});

export const columnCode = css({ bg: "screenDeep" });

export const columnTitle = css({ fontWeight: "extrabold", opacity: "0.7", fontSize: "0.9em" });

export const bubble = css({
  bg: "bubble",
  borderRadius: "0.9em",
  py: "0.7em",
  px: "0.9em",
  width: "max-content",
  maxWidth: "100%",
});

export const field = css({
  position: "relative",
  mt: "auto",
  borderWidth: "0.2em",
  borderStyle: "solid",
  borderColor: "brandBright",
  borderRadius: "0.9em",
  py: "0.8em",
  px: "1em",
  minHeight: "3em",
  fontWeight: "bold",
  boxShadow: "0 0 0 0.35em rgba(78, 141, 255, 0.25)",
});

export const draft = css({
  opacity: "0.55",
  "[data-step='2'] &": { display: "none" },
});

export const typed = css({
  display: "none",
  "[data-step='2'] &": { display: "inline" },
});

export const caret = css({
  display: "inline-block",
  width: "0.18em",
  height: "1.1em",
  bg: "brandBright",
  verticalAlign: "text-bottom",
  ml: "0.2em",
  animation: "blink 1s steps(1) infinite",
});

export const scan = css({
  position: "absolute",
  inset: "-0.7em",
  borderWidth: "0.2em",
  borderStyle: "dashed",
  borderColor: "gold",
  borderRadius: "1.2em",
  opacity: "0",
  transition: ".4s",
  overflow: "hidden",
  "[data-step='1'] &": { opacity: "1" },
  "&::after": {
    content: '""',
    position: "absolute",
    left: "0",
    right: "0",
    height: "0.35em",
    backgroundImage: "linear-gradient(90deg, transparent, var(--colors-gold), transparent)",
    animation: "scan 1.4s ease-in-out infinite",
  },
});

export const scanLabel = css({
  position: "absolute",
  left: "0.6em",
  top: "0.2em",
  bg: "gold",
  color: "ink",
  fontWeight: "black",
  fontSize: "0.85em",
  px: "0.7em",
  py: "0.1em",
  borderRadius: "pill",
  fontStyle: "normal",
});

export const source = css({
  fontFamily: "mono",
  fontWeight: "semibold",
  fontSize: "0.95em",
  lineHeight: "1.8",
  color: "codeBlue",
  whiteSpace: "pre-wrap",
  m: "0",
});

export const keyword = css({ color: "codePink" });

export const cursor = css({
  position: "absolute",
  width: "2.4em",
  height: "2.4em",
  left: "78%",
  top: "90%",
  transition: "left 1s cubic-bezier(.5,0,.2,1), top 1s cubic-bezier(.5,0,.2,1)",
  filter: "drop-shadow(0 0.3em 0.4em rgba(0, 0, 0, 0.4))",
  zIndex: "3",
  "[data-step='0'] &": { left: "30%", top: "78%" },
  "[data-step='1'] &": { left: "32%", top: "82%" },
  "[data-step='2'] &": { left: "32%", top: "82%" },
});

export const cursorRing = css({
  position: "absolute",
  left: "-0.4em",
  top: "-0.4em",
  width: "1.5em",
  height: "1.5em",
  borderRadius: "50%",
  borderWidth: "0.2em",
  borderStyle: "solid",
  borderColor: "white",
  opacity: "0",
  "[data-step='0'] &": { animation: "ring 1.3s infinite" },
});

export const status = css({
  position: "absolute",
  left: "50%",
  bottom: "2.6%",
  transform: "translateX(-50%)",
  bg: "rgba(15, 27, 61, 0.82)",
  py: "0.6em",
  px: "1.6em",
  borderRadius: "pill",
  fontWeight: "extrabold",
  whiteSpace: "nowrap",
});

export const statusText = css({
  display: "none",
  "[data-step='0'] &": { "&[data-at='0']": { display: "inline" } },
  "[data-step='1'] &": { "&[data-at='1']": { display: "inline" } },
  "[data-step='2'] &": { "&[data-at='2']": { display: "inline" } },
});

export const badge = css({
  position: "absolute",
  display: "grid",
  placeItems: "center",
  textAlign: "center",
  width: "84px",
  height: "84px",
  borderRadius: "24px",
  fontWeight: "black",
  lineHeight: "1.1",
  color: "white",
  boxShadow: "0 20px 40px rgba(0, 0, 0, 0.4)",
  transition: ".7s",
  _bp900: { width: "52px", height: "52px", borderRadius: "16px" },
});

export const badgeJa = css({
  right: "-4%",
  top: "-8%",
  bgGradient: "brandSoft",
  transform: "translateZ(110px) scale(0.8)",
  opacity: "0.35",
  "[data-step='2'] &": { opacity: "1", transform: "translateZ(120px) scale(1.1)" },
});

export const badgeEn = css({
  left: "-4%",
  bottom: "6%",
  bgGradient: "accentSoft",
  transform: "translateZ(80px) scale(0.8)",
  opacity: "0.35",
  animation: "bob 5s ease-in-out infinite",
  "[data-step='1'] &": { opacity: "0.7" },
});

export const badgeMark = css({ fontSize: "34px", display: "block", _bp900: { fontSize: "22px" } });
export const badgeLabel = css({ fontSize: "11px", opacity: "0.9", _bp900: { display: "none" } });
