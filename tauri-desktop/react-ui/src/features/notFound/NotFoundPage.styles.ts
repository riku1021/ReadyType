import { css } from "@/styled-system/css";

export const notFoundContainer = css({
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  justifyContent: "center",
  textAlign: "center",
  minHeight: "100%",
});

export const animation = css({
  height: "10rem",
  width: "auto",
  marginBottom: "sm",
});

export const heading = css({
  fontSize: "clamp(27px, 25.18181818181818px + 0.5681818181818182vw, 32px)",
  color: "text.sub",
  marginBottom: "lg",
});

export const paragraph = css({
  fontSize: "clamp(18px, 17.272727272727273px + 0.22727272727272727vw, 20px)",
  color: "text.sub",
  marginBottom: "lg",
});

export const homeLink = css({
  display: "inline-flex!",
  alignItems: "center",
  gap: "xs",
});

export const homeIcon = css({
  fontSize: "1.2rem",
});

export const backButtonWrap = css({
  marginTop: "sm",
});
