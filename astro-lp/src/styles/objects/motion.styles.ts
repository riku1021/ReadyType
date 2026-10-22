import { css } from "styled-system/css";

export const reveal = css({
  ".js &": {
    opacity: "0",
    transform: "translateY(36px)",
    transition: "opacity .8s cubic-bezier(.2,.7,.2,1), transform .8s cubic-bezier(.2,.7,.2,1)",
    "&[data-in='true']": {
      opacity: "1",
      transform: "none",
    },
    "&[data-delay='1']": { transitionDelay: ".12s" },
    "&[data-delay='2']": { transitionDelay: ".24s" },
  },
});

export const tilt = css({
  position: "relative",
  "&::before": {
    content: '""',
    position: "absolute",
    inset: "0",
    borderRadius: "inherit",
    background:
      "radial-gradient(260px circle at var(--sx, 50%) var(--sy, 50%), rgba(124, 92, 255, 0.16), transparent 60%)",
    opacity: "0",
    transition: "opacity .3s",
    pointerEvents: "none",
  },
  _hover: {
    "&::before": { opacity: "1" },
  },
  ".js &": {
    "&[data-in='true']": {
      transition: "transform .18s ease-out, opacity .8s, border-color .25s",
      transitionDelay: "0s",
    },
  },
});
