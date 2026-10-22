import { css } from "styled-system/css";

export const problem = css({
  bgGradient: "problem",
});

export const cards = css({
  display: "grid",
  gridTemplateColumns: "repeat(auto-fit, minmax(min(100%, 320px), 1fr))",
  gap: "clamp(16px, 3vw, 24px)",
});

export const card = css({
  bg: "white",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "line",
  borderBottomWidth: "7px",
  borderRadius: "card",
  p: "clamp(20px, 3vw, 30px)",
  transition: ".25s",
  _hover: { transform: "translateY(-6px) rotate(-0.5deg)" },
  "&:nth-child(2):hover": { transform: "translateY(-6px) rotate(0.5deg)" },
});

export const cardTitle = css({
  fontSize: "1.1875rem",
  lineHeight: "1.35",
  fontWeight: "black",
  mb: "14px",
});

export const bad = css({
  display: "flex",
  alignItems: "center",
  bg: "white",
  borderRadius: "14px",
  px: "18px",
  py: "14px",
  fontFamily: "mono",
  fontWeight: "bold",
  fontSize: "1.1875rem",
  minHeight: "56px",
  boxShadow: "inset 0 0 0 1px var(--colors-line)",
  mb: "12px",
});

export const typed = css({
  display: "inline-block",
  overflow: "hidden",
  whiteSpace: "nowrap",
  maxWidth: "0",
  animation: "typ 3.6s steps(10) infinite alternate",
});

export const miss = css({
  color: "danger",
  fontWeight: "black",
  display: "inline-block",
  animation: "shk 2.6s infinite",
});

export const punch = css({
  textAlign: "center",
  fontSize: "clamp(1.375rem, 3.2vw, 2rem)",
  fontWeight: "black",
  mt: "44px",
  lineHeight: "1.35",
});

export const punchMark = css({ color: "danger" });

export const highlight = css({
  backgroundImage: "linear-gradient(transparent 62%, var(--colors-danger-soft) 62%)",
  px: "6px",
});
