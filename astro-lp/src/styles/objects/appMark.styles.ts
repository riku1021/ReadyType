import { cva } from "styled-system/css";

export const appMark = cva({
  base: {
    width: "26px",
    height: "26px",
    borderRadius: "50%",
    color: "white",
    display: "grid",
    placeItems: "center",
    fontFamily: "latin",
    fontWeight: "extrabold",
    fontSize: "0.6875rem",
    flex: "none",
  },
  variants: {
    tone: {
      vscode: { bg: "app.vscode" },
      cursor: { bg: "app.cursor" },
      terminal: { bg: "app.terminal" },
      slack: { bg: "app.slack" },
      notion: { bg: "app.notion" },
      chrome: { bg: "app.chrome" },
      discord: { bg: "app.discord" },
    },
  },
});
