import { css } from "@/styled-system/css";
import { surfaceMain, surfaceSub } from "@/styles/objects/surface";

/** DialogShell: フルビューポート。dim は dialog 自身、::backdrop は透明 */
export const shell = css({
  position: "fixed",
  inset: "0",
  width: "100%",
  maxWidth: "100%",
  height: "100%",
  maxHeight: "100%",
  margin: "0",
  padding: "0",
  borderWidth: "0",
  backgroundColor: "background.overlay",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  opacity: 0,
  _backdrop: {
    backgroundColor: "transparent",
  },
});

/** Form 用: 端に余白 */
export const formShell = css({
  padding: "sm",
});

export const fadeIn = css({
  animationName: "fadeInOverlay",
  animationDuration: "0.3s",
  animationTimingFunction: "ease-out",
  animationFillMode: "forwards",
});

export const fadeOut = css({
  animationName: "fadeOutOverlay",
  animationDuration: "0.3s",
  animationTimingFunction: "ease-in",
  animationFillMode: "forwards",
});

export const contentFadeIn = css({
  animationName: "fadeInContent",
  animationDuration: "0.3s",
  animationTimingFunction: "ease-out",
  animationFillMode: "forwards",
});

export const contentFadeOut = css({
  animationName: "fadeOutContent",
  animationDuration: "0.3s",
  animationTimingFunction: "ease-in",
  animationFillMode: "forwards",
});

export const closeIcon = css({
  fontSize: "3rem",
  position: "absolute",
  top: "0.5rem",
  right: "0.5rem",
  cursor: "pointer",
  background: "none",
  borderWidth: "0",
  padding: "0",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  color: "text.sub",
  lineHeight: "1",
  transitionProperty: "color",
  transitionDuration: "150ms",
  transitionTimingFunction: "ease",
  _hover: {
    color: "status.error.main",
  },
});

/** Status / Alert / Loading 用パネル */
export const statusPanel = css(surfaceSub, {
  padding: "lg",
  borderRadius: "25px",
  boxShadow: "0 4px 6px {colors.shadow.primary}",
  maxWidth: "500px",
  width: "90%",
  position: "relative",
  opacity: 0,
  transform: "scale(0.9)",
  textAlign: "center",
  overflow: "hidden",
});

export const statusTitle = css({
  fontSize: "1.5rem",
  fontWeight: "semibold",
  marginBottom: "xs",
  color: "text.main",
});

export const statusBody = css({
  fontSize: "1rem",
  color: "text.main",
});

export const statusFooter = css({
  marginTop: "sm",
});

export const animationContainer = css({
  marginInline: "auto",
  marginBottom: "sm",
  display: "flex",
  justifyContent: "center",
  width: "7rem",
  height: "7rem",
});

export const buttonContainer = css({
  display: "flex",
  justifyContent: "center",
  gap: "sm",
  marginTop: "sm",
});

export const loadingContainer = css({
  marginInline: "auto",
  marginBottom: "sm",
  display: "flex",
  justifyContent: "center",
  alignItems: "center",
  width: "8rem",
  height: "8rem",
});

export const progressBarContainer = css({
  position: "absolute",
  bottom: "0",
  left: "0",
  right: "0",
  height: "5px",
  overflow: "hidden",
});

export const progressBarStyle = css(surfaceMain, {
  position: "absolute",
  bottom: "0",
  left: "0",
  width: "100%",
  height: "100%",
  transformOrigin: "left",
  animationName: "alertProgressBar",
  animationTimingFunction: "linear",
  animationFillMode: "forwards",
});

/** Form 用パネル */
export const formPanel = css(surfaceSub, {
  padding: "lg",
  borderRadius: "25px",
  boxShadow: "0 4px 6px {colors.shadow.primary}",
  maxWidth: "520px",
  width: "100%",
  maxHeight: "90vh",
  overflowY: "auto",
  position: "relative",
  opacity: 0,
  transform: "scale(0.9)",
  textAlign: "left",
});

export const formTitle = css({
  fontSize: "1.5rem",
  fontWeight: "semibold",
  marginBottom: "md",
  paddingRight: "2.5rem",
  color: "text.main",
});

export const formBody = css({
  color: "text.main",
});

export const formFooter = css({
  marginTop: "md",
});
