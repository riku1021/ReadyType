import { css } from "styled-system/css";

export const floatingCta = css({
  position: "fixed",
  left: "50%",
  bottom: "22px",
  transform: "translate(-50%, 120px)",
  zIndex: "cta",
  bgGradient: "brand",
  color: "white",
  fontWeight: "black",
  px: "28px",
  py: "14px",
  borderRadius: "pill",
  boxShadow: "cta",
  transition: "transform .45s cubic-bezier(.2,.7,.2,1)",
  whiteSpace: "nowrap",
  maxWidth: "calc(100% - 32px)",
  "&[data-show='true']": { transform: "translate(-50%, 0)" },
  _hover: { filter: "brightness(1.08)" },
});
