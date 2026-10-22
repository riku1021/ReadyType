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
  height: "68px",
});

export const navList = css({
  display: "flex",
  gap: "26px",
  listStyle: "none",
  fontWeight: "extrabold",
  fontSize: "14.5px",
  color: "muted",
  _bp900: { display: "none" },
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
