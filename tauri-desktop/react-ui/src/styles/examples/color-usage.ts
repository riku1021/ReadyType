import { css } from "@/styled-system/css";
import { token } from "@/styled-system/tokens";
/**
 * カラートークンの使用例
 * @description PandaCSSで実装したカラートークンの使用方法を示すサンプル
 */

// CSS関数を使用したスタイル定義例
export const exampleStyles = {
  // ブランドカラーの使用例
  primaryButton: css({
    backgroundColor: token("colors.brand.main"),
    color: "white",
    border: "2px solid",
    borderColor: token("colors.brand.main"),
    "&:hover": {
      backgroundColor: token("colors.brand.hover"),
    },
  }),

  secondaryButton: css({
    backgroundColor: token("colors.brand.main"),
    color: "white",
    "&:hover": {
      backgroundColor: token("colors.brand.hover"),
    },
  }),

  // テーマ対応カラーの使用例
  card: css({
    backgroundColor: token("colors.background.sub"),
    color: "text.main",
    border: "1px solid",
    borderColor: "border.primary",
    boxShadow: "0 2px 4px",
    boxShadowColor: "shadow.primary",
    "&:hover": {
      backgroundColor: token("colors.background.hover"),
    },
  }),

  // 状態カラーの使用例
  successMessage: css({
    backgroundColor: token("colors.status.success.main"),
    color: "white",
    padding: "1rem",
    borderRadius: "0.5rem",
  }),

  errorMessage: css({
    backgroundColor: token("colors.status.error.main"),
    color: "white",
    padding: "1rem",
    borderRadius: "0.5rem",
  }),

  // アイコンスタイル例
  icon: css({
    color: token("colors.icon.primary"),
    "&:disabled": {
      color: token("colors.icon.disabled"),
    },
  }),

  // テキストスタイル例
  heading: css({
    color: token("colors.text.main"),
    fontSize: "2xl",
    fontWeight: "bold",
  }),

  subtext: css({
    color: token("colors.text.sub"),
    fontSize: "sm",
  }),

  placeholder: css({
    color: token("colors.text.placeholder"),
    fontStyle: "italic",
  }),
};

// token関数を使用した値の取得例
export const colorValues = {
  // ブランドカラーの値を取得
  primaryColor: "colors.brand.main",
  primaryLight: "colors.brand.light",
  primaryHover: "colors.brand.hover",

  // 状態カラーの値を取得
  successColor: "colors.status.success.main",
  successSoft: "colors.status.success.soft",
  successSoftBorder: "colors.status.success.softBorder",
  warningColor: "colors.status.warning.main",
  errorColor: "colors.status.error.main",
  attentionColor: "colors.status.attention.main",

  // テーマ対応カラーの値を取得
  textMain: "colors.text.main",
  backgroundMain: "colors.background.main",
  borderPrimary: "colors.border.primary",
};

// 動的スタイル生成の例
export const createButtonStyle = (variant: "primary" | "secondary" | "attention") => {
  const variantColors = {
    primary: {
      bg: token("colors.brand.main"),
      hover: token("colors.brand.hover"),
    },
    secondary: {
      bg: token("colors.brand.main"),
      hover: token("colors.brand.hover"),
    },
    attention: {
      bg: token("colors.status.attention.main"),
      hover: token("colors.status.attention.main"),
    },
  };

  return css({
    backgroundColor: variantColors[variant].bg,
    color: "white",
    padding: "0.75rem 1.5rem",
    borderRadius: "0.5rem",
    border: "none",
    cursor: "pointer",
    transition: "all 0.2s ease",
    "&:hover": {
      backgroundColor: variantColors[variant].hover,
      transform: "translateY(-1px)",
    },
    "&:active": {
      transform: "translateY(0)",
    },
    "&:disabled": {
      backgroundColor: token("colors.text.disabled"),
      cursor: "not-allowed",
      transform: "none",
    },
  });
};

// テーマ切り替えに対応したスタイル例
export const themeAwareStyles = {
  container: css({
    backgroundColor: token("colors.background.main"),
    color: "text.main",
    minHeight: "100vh",
    transition: "background-color 0.3s ease, color 0.3s ease",
  }),

  sidebar: css({
    backgroundColor: token("colors.background.sub"),
    borderRight: "1px solid",
    borderColor: "border.primary",
    padding: "1rem",
  }),

  overlay: css({
    backgroundColor: token("colors.background.overlay"),
    position: "fixed",
    inset: 0,
    zIndex: 50,
  }),
};
