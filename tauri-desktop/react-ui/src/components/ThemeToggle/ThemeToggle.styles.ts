import { css } from "@/styled-system/css";

export const button = css({
  position: "relative",
  width: "1.75rem",
  height: "1.75rem",
  background: "transparent",
  border: "none",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: "0",
});

const iconBase = {
  position: "absolute" as const,
  width: "1.75rem",
  height: "1.75rem",
  color: "icon.secondary",
  transitionProperty: "transform, opacity, color",
  transitionDuration: "250ms",
  transitionTimingFunction: "ease",
};

export const iconSun = css({
  ...iconBase,
  transform: "rotate(0deg) scale(1)",
  opacity: 1,
  _dark: {
    transform: "rotate(-90deg) scale(0)",
    opacity: 0,
  },
});

export const iconMoon = css({
  ...iconBase,
  transform: "rotate(90deg) scale(0)",
  opacity: 0,
  _dark: {
    transform: "rotate(0deg) scale(1)",
    opacity: 1,
  },
});
