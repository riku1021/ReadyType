import { css } from "@/styled-system/css";

/** Drawer ヘッダーと同じロゴ行（高さ・字間・アクセント） */
export const logoRow = css({
  display: "flex",
  alignItems: "center",
  height: "60px",
  flexShrink: 0,
  color: "text.main",
});

export const logoImageBox = css({
  width: "60px",
  height: "60px",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  flexShrink: 0,
});

export const logoImage = css({
  height: "7",
  width: "auto",
});

export const logoText = css({
  display: "flex",
  alignItems: "center",
  whiteSpace: "nowrap",
  fontSize: "2xl",
  fontWeight: "semibold",
  lineHeight: "none",
});

export const logoAccent = css({
  color: "brand.main",
});
