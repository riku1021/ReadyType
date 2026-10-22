import { css } from "styled-system/css";

export const header = css({
  position: "sticky",
  top: "0",
  zIndex: "header",
  backgroundColor: "rgba(246, 248, 253, 0.85)",
  backdropFilter: "blur(12px)",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "line",
  transition: "box-shadow .3s",
  "&[data-scrolled='true']": { boxShadow: "header" },
});

export const nav = css({
  display: "flex",
  alignItems: "center",
  justifyContent: "space-between",
  gap: "clamp(8px, 2vw, 24px)",
  height: "auto",
  minHeight: "68px",
  flexWrap: "wrap",
  rowGap: "6px",
  py: "6px",
});

export const navList = css({
  display: "flex",
  gap: "clamp(14px, 2.2vw, 26px)",
  listStyle: "none",
  fontWeight: "extrabold",
  fontSize: "0.9062rem",
  color: "muted",
  _menu: { display: "none" },
});

export const menuButton = css({
  display: "none",
  width: "44px",
  height: "44px",
  border: "0",
  borderRadius: "14px",
  bg: "white",
  boxShadow: "inset 0 0 0 1px var(--colors-line)",
  placeItems: "center",
  cursor: "pointer",
  flex: "none",
  _menu: { display: "grid" },
  "& i, & i::before, & i::after": {
    display: "block",
    width: "18px",
    height: "2px",
    borderRadius: "2px",
    bg: "ink",
    transition: ".25s",
    content: '""',
    position: "relative",
  },
  "& i::before": { position: "absolute", top: "-6px" },
  "& i::after": { position: "absolute", top: "6px" },
  "&[aria-expanded='true'] i": { backgroundColor: "transparent" },
  "&[aria-expanded='true'] i::before": { top: "0", transform: "rotate(45deg)" },
  "&[aria-expanded='true'] i::after": { top: "0", transform: "rotate(-45deg)" },
});

export const mobileNav = css({
  display: "none",
  position: "absolute",
  left: "0",
  right: "0",
  top: "100%",
  pt: "8px",
  pb: "20px",
  px: "gutter",
  bg: "canvas",
  backdropFilter: "blur(12px)",
  borderBottomWidth: "1px",
  borderBottomStyle: "solid",
  borderBottomColor: "line",
  boxShadow: "header",
  "&[data-open='true']": { display: "grid" },
  "& a": {
    display: "flex",
    alignItems: "center",
    minHeight: "48px",
    fontWeight: "extrabold",
    borderBottomWidth: "1px",
    borderBottomStyle: "solid",
    borderBottomColor: "line",
  },
  "& a:last-child": { borderBottomWidth: "0", color: "brand" },
});

export const navLink = css({
  position: "relative",
  py: "4px",
  "&::after": {
    content: '""',
    position: "absolute",
    left: "0",
    bottom: "-2px",
    height: "3px",
    width: "0",
    borderRadius: "3px",
    bgGradient: "brand",
    transition: ".25s",
  },
  _hover: {
    color: "brand",
    "&::after": { width: "100%" },
  },
});
