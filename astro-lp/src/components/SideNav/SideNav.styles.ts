import { css } from "styled-system/css";

export const sideNav = css({
  position: "fixed",
  right: "18px",
  top: "50%",
  transform: "translateY(-50%)",
  zIndex: "dots",
  display: "none",
  gap: "12px",
  _wide: { display: "grid" },
});

export const sideLink = css({
  position: "relative",
  width: "12px",
  height: "12px",
  borderRadius: "50%",
  bg: "white",
  borderWidth: "2px",
  borderStyle: "solid",
  borderColor: "#A9B7DD",
  boxShadow: "0 2px 8px rgba(40, 70, 160, 0.2)",
  transition: ".25s",
  "&[data-current='true']": {
    bg: "brand",
    borderColor: "brand",
    transform: "scale(1.35)",
  },
  "& span": {
    position: "absolute",
    right: "22px",
    top: "50%",
    transform: "translateY(-50%) translateX(6px)",
    bg: "ink",
    color: "white",
    fontSize: "0.75rem",
    fontWeight: "extrabold",
    px: "10px",
    py: "3px",
    borderRadius: "pill",
    whiteSpace: "nowrap",
    opacity: "0",
    pointerEvents: "none",
    transition: ".2s",
  },
  _hover: {
    "& span": { opacity: "1", transform: "translateY(-50%)" },
  },
});
