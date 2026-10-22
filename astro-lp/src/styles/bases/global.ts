import { defineGlobalStyles } from "@pandacss/dev";

export const globalCss = defineGlobalStyles({
  html: {
    scrollBehavior: "smooth",
    textSizeAdjust: "100%",
  },
  "h1, h2, h3, h4, h5, p, ul, ol": {
    margin: "0",
    padding: "0",
  },
  "h1, h2, h3": {
    wordBreak: "auto-phrase",
  },
  p: {
    textWrap: "pretty",
  },
  svg: {
    maxWidth: "100%",
  },
  body: {
    fontFamily: "body",
    backgroundColor: "canvas",
    color: "ink",
    lineHeight: "1.8",
    fontWeight: "medium",
    overflowX: "hidden",
    overflowWrap: "break-word",
  },
  a: {
    color: "inherit",
    textDecoration: "none",
  },
  "a:focus-visible, button:focus-visible, summary:focus-visible, [tabindex]:focus-visible": {
    outline: "3px solid #1A6BFF",
    outlineOffset: "3px",
    borderRadius: "8px",
  },
});
