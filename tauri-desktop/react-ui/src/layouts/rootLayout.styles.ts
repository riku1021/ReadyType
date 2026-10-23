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
  display: "flex",
  flexDirection: "column",
  alignItems: "center",
  minWidth: "0",
  minHeight: "0",
  overflow: "auto",
  padding: "sm",
  boxSizing: "border-box",
  // スライドのスナップショットを不透明にする。body と同じ面なので静止時の見た目は変わらない。
  backgroundColor: "background.main",
  // styles/bases/pageSlide.ts の pageSlideMainName と揃える
  viewTransitionName: "page-slide-main",
});

/** メイン窓のページ共通。幅いっぱい＋水平中央。各ページでは指定しない。 */
export const contentColumn = css({
  width: "100%",
  marginInline: "auto",
  minWidth: "0",
});
