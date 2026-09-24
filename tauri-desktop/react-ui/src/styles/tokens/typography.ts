/**
 * タイポグラフィトークン定義
 * フォントサイズ、ウェイト、行間などのタイポグラフィシステム
 */

import { fluidType } from "../utils/fluid-utils";

// フォントウェイト定義
export const fontWeights = {
  thin: 100,
  extralight: 200,
  regular: 300,
  normal: 400,
  medium: 500,
  semibold: 600,
  bold: 700,
} as const;

// 型定義
export type FontWeightType = keyof typeof fontWeights;

/**
 * フォントウェイト値を取得する関数
 * @param key - フォントウェイトのキー
 * @returns フォントウェイト値
 */
export const getFontWeight = (key: FontWeightType): number => {
  return fontWeights[key];
};

// 流体タイプ（レスポンシブフォントサイズ）の定義
export const fluidTypography = {
  // 見出し
  h1: fluidType(32, 40),
  h2: fluidType(26, 32),
  h3: fluidType(22, 28),
  h4: fluidType(20, 24),
  h5: fluidType(18, 20),
  h6: fluidType(16, 18),

  // テキストサイズ
  textSmall: fluidType(12, 14),
  textBase: fluidType(16, 18),
  textLarge: fluidType(18, 20),

  // 大きな見出し
  display: fluidType(40, 56),
} as const;

// 行間（line-height）の定義
export const lineHeights = {
  tight: 1.2,
  normal: 1.5,
  relaxed: 1.75,
  loose: 2,
} as const;

// 文字間隔（letter-spacing）の定義
export const letterSpacings = {
  tighter: "-0.05em",
  tight: "-0.025em",
  normal: "0em",
  wide: "0.025em",
  wider: "0.05em",
  widest: "0.1em",
} as const;

// フォントファミリーの定義
export const fontFamilies = {
  sans: [
    "Noto Sans JP",
    "Inter",
    "system-ui",
    "-apple-system",
    "BlinkMacSystemFont",
    "Segoe UI",
    "Roboto",
    "Helvetica Neue",
    "Arial",
    "sans-serif",
  ],
  serif: ["Georgia", "Cambria", "Times New Roman", "Times", "serif"],
  mono: [
    "Fira Code",
    "SFMono-Regular",
    "Menlo",
    "Monaco",
    "Consolas",
    "Liberation Mono",
    "Courier New",
    "monospace",
  ],
} as const;

/**
 * Panda CSS用のタイポグラフィトークン
 * styled-systemで使用するためのトークン形式
 */
export const typographyTokens = {
  fontSizes: {
    xs: { value: "0.75rem" },
    sm: { value: "0.875rem" },
    base: { value: "1rem" },
    lg: { value: "1.125rem" },
    xl: { value: "1.25rem" },
    "2xl": { value: "1.5rem" },
    "3xl": { value: "1.875rem" },
    "4xl": { value: "2.25rem" },
    "5xl": { value: "3rem" },
    "6xl": { value: "3.75rem" },
    "7xl": { value: "4.5rem" },
    "8xl": { value: "6rem" },
    "9xl": { value: "8rem" },
  },
  fontWeights: {
    thin: { value: fontWeights.thin },
    extralight: { value: fontWeights.extralight },
    regular: { value: fontWeights.regular },
    normal: { value: fontWeights.normal },
    medium: { value: fontWeights.medium },
    semibold: { value: fontWeights.semibold },
    bold: { value: fontWeights.bold },
  },
  lineHeights: {
    none: { value: 1 },
    tight: { value: lineHeights.tight },
    snug: { value: 1.375 },
    normal: { value: lineHeights.normal },
    relaxed: { value: lineHeights.relaxed },
    loose: { value: lineHeights.loose },
  },
  letterSpacings: {
    tighter: { value: letterSpacings.tighter },
    tight: { value: letterSpacings.tight },
    normal: { value: letterSpacings.normal },
    wide: { value: letterSpacings.wide },
    wider: { value: letterSpacings.wider },
    widest: { value: letterSpacings.widest },
  },
  fontFamilies: {
    sans: { value: fontFamilies.sans.join(", ") },
    serif: { value: fontFamilies.serif.join(", ") },
    mono: { value: fontFamilies.mono.join(", ") },
  },
} as const;

/** Panda `theme.tokens` に載せるタイポ（`fonts` が正規キー） */
export const pandaTypographyTokens = {
  fontSizes: typographyTokens.fontSizes,
  fontWeights: typographyTokens.fontWeights,
  lineHeights: typographyTokens.lineHeights,
  letterSpacings: typographyTokens.letterSpacings,
  fonts: typographyTokens.fontFamilies,
} as const;

// デフォルトエクスポート
export default typographyTokens;
