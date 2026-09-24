/**
 * 参考用のリセット定義。実行時は Panda の `preflight: true` が担当する。
 * このオブジェクトは `panda.config.ts` に登録しない（二重リセットになるため）。
 */

// ==============================================
// リセットスタイル
// ==============================================

export const reset = {
  // ============================================
  // Boxモデルとボーダーのリセット
  // ============================================
  "*, ::before, ::after": {
    boxSizing: "border-box",
    borderStyle: "solid",
    borderWidth: "0",
    minWidth: "0",
  },

  // ============================================
  // ドキュメント設定
  // ============================================
  html: {
    lineHeight: "1.15",
    WebkitTextSizeAdjust: "100%",
    WebkitTapHighlightColor: "transparent",
  },

  // ============================================
  // セクションの初期化
  // ============================================
  body: { margin: "0" },
  main: { display: "block" },

  // ============================================
  // 垂直リズムの統一
  // ============================================
  "p, table, blockquote, address, pre, iframe, form, figure, dl": {
    margin: "0",
  },

  // ============================================
  // 見出しの初期化
  // ============================================
  "h1, h2, h3, h4, h5, h6": {
    fontSize: "inherit",
    fontWeight: "inherit",
    margin: "0",
  },

  // ============================================
  // 箇条書きリストのリセット
  // ============================================
  "ul, ol": {
    margin: "0",
    padding: "0",
    listStyle: "none",
  },

  // ============================================
  // 定義リストのリセット
  // ============================================
  dt: { fontWeight: "bold" },
  dd: { marginLeft: "0" },

  // ============================================
  // コンテンツのグループ化
  // ============================================
  hr: {
    boxSizing: "content-box",
    height: "0",
    overflow: "visible",
    borderTopWidth: "1px",
    margin: "0",
    clear: "both",
    color: "inherit",
  },
  pre: {
    fontFamily: "monospace, monospace",
    fontSize: "inherit",
  },
  address: { fontStyle: "inherit" },

  // ============================================
  // テキストレベルの意味付け要素のリセット
  // ============================================
  a: {
    backgroundColor: "transparent",
    textDecoration: "none",
    color: "inherit",
  },
  "abbr[title]": { textDecoration: "underline dotted" },
  "b, strong": { fontWeight: "bolder" },
  "code, kbd, samp": {
    fontFamily: "monospace, monospace",
    fontSize: "inherit",
  },
  small: { fontSize: "80%" },
  "sub, sup": {
    fontSize: "75%",
    lineHeight: "0",
    position: "relative",
    verticalAlign: "baseline",
  },
  sub: { bottom: "-0.25em" },
  sup: { top: "-0.5em" },
};
