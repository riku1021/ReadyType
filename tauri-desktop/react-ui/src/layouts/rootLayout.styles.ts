import { css } from "@/styled-system/css";

export const rootLayout = css({
  display: "grid",
  gridTemplateRows: "auto 1fr",
  gridTemplateColumns: "auto 1fr",
  gridTemplateAreas: `
    "drawer header"
    "drawer main"
  `,
  height: "100vh",
  overflow: "hidden",
});

export const mainContent = css({
  gridArea: "main",
  minWidth: "0",
  minHeight: "0",
  overflow: "auto",
  padding: "sm",
  boxSizing: "border-box",
});
