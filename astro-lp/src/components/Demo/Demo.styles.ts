import { css } from "styled-system/css";

export const frame = css({
  maxWidth: "900px",
  mx: "auto",
  borderRadius: "32px",
  p: "10px",
  bgGradient: "brand",
  boxShadow: "video",
  position: "relative",
  width: "100%",
  containerType: "inline-size",
  transition: "transform .4s",
  _hover: { transform: "scale(1.015)" },
  "&::before": {
    content: '""',
    position: "absolute",
    inset: "-14px",
    borderRadius: "44px",
    borderWidth: "2px",
    borderStyle: "dashed",
    borderColor: "rgba(124, 92, 255, 0.35)",
    animation: "orbit 40s linear infinite reverse",
  },
});

export const screen = css({
  position: "relative",
  borderRadius: "24px",
  overflow: "hidden",
  aspectRatio: "16 / 9",
  bg: "ink",
  _frameTight: { aspectRatio: "4 / 3" },
});

export const demo = css({
  position: "absolute",
  inset: "0",
  containerType: "inline-size",
  bgGradient: "demo",
  fontSize: "1rem",
  color: "white",
});

export const demoScale = css({
  position: "absolute",
  inset: "0",
  fontSize: "max(0.45rem, 1.55cqw)",
});

export const menu = css({
  display: "flex",
  gap: "1.6em",
  alignItems: "center",
  py: "0.55em",
  px: "1.4em",
  bg: "rgba(15, 27, 61, 0.55)",
  backdropFilter: "blur(8px)",
  fontWeight: "bold",
});

export const menuDot = css({
  width: "0.9em",
  height: "0.9em",
  borderRadius: "50%",
  bg: "white",
});

export const menuSpacer = css({ flex: "1" });

export const ime = css({
  minWidth: "2.6em",
  textAlign: "center",
  borderRadius: "0.6em",
  px: "0.6em",
  py: "0.1em",
  fontWeight: "black",
  color: "white",
  transition: ".3s",
  "&[data-ime='ja']": { bg: "brand" },
  "&[data-ime='en']": { bg: "accent" },
  "&[data-flash='true']": {
    transform: "scale(1.35)",
    boxShadow: "0 0 0 0.5em rgba(255, 255, 255, 0.35)",
  },
});

export const window = css({
  position: "absolute",
  left: "6%",
  right: "6%",
  top: "13%",
  bottom: "15%",
  bg: "screen",
  borderRadius: "1.4em",
  boxShadow: "0 2em 5em rgba(15, 27, 61, 0.5)",
  overflow: "hidden",
});

export const dots = css({ display: "flex", gap: "0.5em", py: "0.9em", px: "1.1em" });

export const dot = css({
  width: "0.9em",
  height: "0.9em",
  borderRadius: "50%",
  "&:nth-child(1)": { bg: "#FF5F57" },
  "&:nth-child(2)": { bg: "#FEBC2E" },
  "&:nth-child(3)": { bg: "#28C840" },
});

export const panes = css({
  display: "grid",
  gridTemplateColumns: "0.85fr 1.15fr",
  height: "calc(100% - 2.7em)",
});

export const pane = css({
  py: "1em",
  px: "1.4em",
  borderRightWidth: "1px",
  borderRightStyle: "solid",
  borderRightColor: "rgba(255, 255, 255, 0.1)",
  position: "relative",
  display: "flex",
  flexDirection: "column",
  gap: "0.9em",
  transition: ".3s",
  "&[data-active='true']": { bg: "paneOn" },
  "&[data-surface='code']": { border: "0", bg: "screenDeep" },
  "&[data-surface='code'][data-active='true']": { bg: "paneOnDeep" },
});

export const paneTitle = css({ fontWeight: "extrabold", opacity: "0.75", fontSize: "0.9em" });

export const bubble = css({
  bg: "bubble",
  borderRadius: "1em",
  py: "0.8em",
  px: "1em",
  width: "max-content",
  maxWidth: "100%",
});

export const demoField = css({
  position: "relative",
  mt: "auto",
  borderWidth: "0.2em",
  borderStyle: "solid",
  borderColor: "fieldLine",
  borderRadius: "1em",
  py: "0.9em",
  px: "1.1em",
  lineHeight: "1.8",
  minHeight: "4em",
  transition: ".3s",
  fontWeight: "bold",
  "[data-active='true'] &": {
    borderColor: "brandBright",
    boxShadow: "0 0 0 0.35em rgba(78, 141, 255, 0.25)",
  },
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
  "[data-scan='true'] &": { opacity: "1" },
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

export const codeStage = css({
  position: "relative",
});

export const code = css({
  fontFamily: "mono",
  fontWeight: "semibold",
  fontSize: "1em",
  lineHeight: "1.9",
  color: "codeBlue",
  whiteSpace: "pre-wrap",
  m: "0",
});

export const keyword = css({ color: "codePink" });

export const demoCaret = css({
  bg: "brandBright",
  display: "none",
  width: "0.18em",
  height: "1.1em",
  ml: "0.15em",
  verticalAlign: "text-bottom",
  animation: "blink 1s steps(1) infinite",
  "[data-active='true'] &": { display: "inline-block" },
});

export const demoCaretCode = css({
  bg: "gold",
  display: "none",
  width: "0.18em",
  height: "1.1em",
  verticalAlign: "text-bottom",
  animation: "blink 1s steps(1) infinite",
  "[data-active='true'] &": { display: "inline-block" },
});

export const pointer = css({
  position: "absolute",
  width: "2.6em",
  height: "2.6em",
  left: "88%",
  top: "92%",
  transition: "left 1s cubic-bezier(.5,0,.2,1), top 1s cubic-bezier(.5,0,.2,1)",
  zIndex: "5",
  filter: "drop-shadow(0 0.3em 0.4em rgba(0, 0, 0, 0.35))",
});

export const pointerRing = css({
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
  "[data-click='true'] &": { animation: "ring .6s" },
});

export const toast = css({
  position: "absolute",
  right: "2.2%",
  top: "9%",
  bg: "white",
  color: "ink",
  fontWeight: "black",
  borderRadius: "1em",
  py: "0.7em",
  px: "1.2em",
  boxShadow: "0 1em 2.4em rgba(15, 27, 61, 0.3)",
  opacity: "0",
  transform: "translateY(-0.8em)",
  transition: ".35s",
  zIndex: "6",
  "&[data-show='true']": { opacity: "1", transform: "none" },
});

export const captionBar = css({
  position: "absolute",
  left: "50%",
  bottom: "3.4%",
  transform: "translateX(-50%)",
  bg: "rgba(15, 27, 61, 0.78)",
  backdropFilter: "blur(6px)",
  py: "0.7em",
  px: "1.8em",
  borderRadius: "pill",
  fontWeight: "extrabold",
  fontSize: "1.15em",
  whiteSpace: "nowrap",
});

export const timeline = css({
  position: "relative",
  maxWidth: "900px",
  mx: "auto",
  mt: "34px",
  display: "grid",
  gridTemplateColumns: "repeat(3, 1fr)",
  gap: "12px",
  "&::before": {
    content: '""',
    position: "absolute",
    left: "16.66%",
    right: "16.66%",
    top: "16.5px",
    height: "5px",
    borderRadius: "5px",
    bg: "track",
  },
});

export const timelineFill = css({
  position: "absolute",
  left: "16.66%",
  top: "16.5px",
  height: "5px",
  borderRadius: "5px",
  bgGradient: "brand",
  width: "calc(66.66% * var(--f, 0))",
  transition: "width 1s cubic-bezier(.5,0,.2,1)",
  zIndex: "1",
});

export const step = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  textAlign: "center",
  gap: "0",
  opacity: "0.7",
  position: "relative",
  zIndex: "2",
  transition: "opacity .3s",
  "&[data-state='on'], &[data-state='done']": { opacity: "1" },
});

export const stepBadge = css({
  position: "relative",
  width: "38px",
  height: "38px",
  borderRadius: "50%",
  bg: "white",
  borderWidth: "3px",
  borderStyle: "solid",
  borderColor: "#C7D3F0",
  color: "brand",
  display: "grid",
  placeItems: "center",
  fontFamily: "latin",
  fontWeight: "extrabold",
  fontSize: "0.9375rem",
  mb: "10px",
  transition: ".4s",
  "&::before": {
    content: '""',
    position: "absolute",
    inset: "0",
    borderRadius: "inherit",
    bgGradient: "brand",
    opacity: "0",
    transition: "opacity .4s",
  },
  "& > *": { position: "relative", zIndex: "1" },
  "[data-state='on'] &": {
    borderColor: "white",
    color: "white",
    boxShadow: "0 0 0 6px rgba(124, 92, 255, 0.2)",
    animation: "stepPulse 2s infinite",
    "&::before": { opacity: "1" },
  },
  "[data-state='done'] &": {
    borderColor: "white",
    color: "white",
    "&::before": { opacity: "1" },
  },
});

export const stepNum = css({
  "[data-state='done'] &": { display: "none" },
});

export const stepCheck = css({
  "[data-demo-step] &": { display: "none" },
  "[data-demo-step][data-state='done'] &": { display: "inline-block" },
});

export const stepLabel = css({
  fontWeight: "extrabold",
  fontSize: "clamp(0.75rem, 1.6vw, 0.875rem)",
  lineHeight: "1.5",
});

export const stepHint = css({
  display: "block",
  fontWeight: "semibold",
  fontSize: "0.75rem",
  color: "muted",
});
