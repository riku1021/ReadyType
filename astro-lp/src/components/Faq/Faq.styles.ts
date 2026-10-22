import { css } from "styled-system/css";

export const list = css({
  maxWidth: "820px",
  mx: "auto",
  display: "grid",
  gap: "12px",
});

export const item = css({
  bg: "white",
  borderWidth: "1px",
  borderStyle: "solid",
  borderColor: "line",
  borderBottomWidth: "5px",
  borderRadius: "18px",
  px: "24px",
  transition: ".25s",
  _hover: { borderColor: "hoverLine" },
  "&[open]": { borderColor: "brand", boxShadow: "faq" },
  "&::details-content": {
    display: "grid",
    gridTemplateRows: "0fr",
    transition: "grid-template-rows .3s ease, content-visibility .3s allow-discrete",
  },
  "&[open]::details-content": { gridTemplateRows: "1fr" },
  _motionReduce: { "&::details-content": { transition: "none" } },
});

export const summary = css({
  listStyle: "none",
  cursor: "pointer",
  py: "20px",
  fontWeight: "extrabold",
  display: "flex",
  justifyContent: "space-between",
  gap: "16px",
  "&::-webkit-details-marker": { display: "none" },
  "[open] &": { color: "brand" },
});

export const mark = css({
  position: "relative",
  alignSelf: "center",
  flexShrink: "0",
  width: "18px",
  height: "18px",
  transition: "transform .6s cubic-bezier(0.34, 1.56, 0.64, 1)",
  "&::before, &::after": {
    content: '""',
    position: "absolute",
    top: "50%",
    left: "50%",
    transform: "translate(-50%, -50%)",
    borderRadius: "10px",
    background: "brand",
    display: "block",
  },
  "&::before": { width: "75%", height: "3px" },
  "&::after": {
    width: "3px",
    height: "75%",
    transition: "height .3s cubic-bezier(0.76, 0, 0.24, 1)",
  },
  "[open] &": {
    transform: "rotate(180deg)",
    "&::after": { height: "0" },
  },
  _motionReduce: {
    transition: "none",
    "&::after": { transition: "none" },
  },
});

export const answer = css({ overflow: "hidden", minHeight: "0" });

export const answerBody = css({ pb: "20px", color: "muted" });
