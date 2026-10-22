import { defineGlobalStyles } from "@pandacss/dev";

export const globalCss = defineGlobalStyles({
  html: {
    scrollBehavior: "smooth",
  },
  "h1, h2, h3, h4, h5, p, ul, ol": {
    margin: "0",
    padding: "0",
  },
  body: {
    fontFamily: "body",
    backgroundColor: "canvas",
    color: "ink",
    lineHeight: "1.8",
    fontWeight: "medium",
    overflowX: "hidden",
  },
  a: {
    color: "inherit",
    textDecoration: "none",
  },
});
