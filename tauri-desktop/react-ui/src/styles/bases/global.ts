/**
 * html / body などの全体スタイル。
 * `css()` は使わない（styled-system は config から生成されるため）。
 */
export const globalCss = {
  html: {
    minHeight: "100%",
    backgroundColor: "background.main",
    color: "text.main",
    // Panda focusRing ユーティリティのデフォルト色
    "--global-color-focus-ring": "colors.brand.main",
    // 既定面は background.main（子孫 Button が継承）
    "--button-surface-shadow": "{colors.shadow.primary}",
    "--button-framed-border-width": "1px",
    "--button-primary-border-width": "1px",
  },
  body: {
    margin: "0",
    minHeight: "100vh",
    fontFamily: "sans",
    backgroundColor: "background.main",
    color: "text.main",
    // html から継承されるが、body 単体でも面コンテキストが揃うように明示
    "--button-surface-shadow": "{colors.shadow.primary}",
    "--button-framed-border-width": "1px",
    "--button-primary-border-width": "1px",
  },
  "#root": {
    minHeight: "100vh",
  },
  /** ブラウザのデフォルト focus outline（外側リング）を消す。枠線色は各コントロールの `_focus` で変える。 */
  "*:focus": {
    outline: "none",
  },
  "html.enable-transitions": {
    transitionProperty: "background-color, color, border-color, box-shadow",
    transitionDuration: "200ms",
    transitionTimingFunction: "ease",
  },
  "body.no-scroll": {
    overflow: "hidden",
  },
};
