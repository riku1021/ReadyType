/**
 * タイポグラフィ使用例
 * フォントサイズ、ウェイト、行間などの使用方法とベストプラクティスを示す
 */

import { css } from "@/styled-system/css";

import {
  type FontWeightType,
  fluidTypography,
  fontFamilies,
  fontWeights,
  getFontWeight,
  letterSpacings,
  lineHeights,
} from "../tokens/typography";

/**
 * タイポグラフィ使用例のスタイル定義
 */

// 1. 見出し要素のスタイル定義
export const headingStyles = {
  h1: css({
    fontSize: fluidTypography.h1,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),

  h2: css({
    fontSize: fluidTypography.h2,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),

  h3: css({
    fontSize: fluidTypography.h3,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),

  h4: css({
    fontSize: fluidTypography.h4,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),

  h5: css({
    fontSize: fluidTypography.h5,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),

  h6: css({
    fontSize: fluidTypography.h6,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.5em",
  }),
};

// 2. テキスト要素のスタイル定義
export const textStyles = {
  paragraph: css({
    fontSize: fluidTypography.textBase,
    fontWeight: fontWeights.normal,
    lineHeight: lineHeights.normal,
    marginBottom: "1rem",
  }),

  small: css({
    fontSize: fluidTypography.textSmall,
    fontWeight: fontWeights.normal,
    lineHeight: lineHeights.normal,
  }),

  large: css({
    fontSize: fluidTypography.textLarge,
    fontWeight: fontWeights.normal,
    lineHeight: lineHeights.normal,
  }),

  display: css({
    fontSize: fluidTypography.display,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    letterSpacing: letterSpacings.tight,
  }),
};

// 3. フォントファミリー別のスタイル
export const fontFamilyStyles = {
  sans: css({
    fontFamily: fontFamilies.sans.join(", "),
  }),

  serif: css({
    fontFamily: fontFamilies.serif.join(", "),
  }),

  mono: css({
    fontFamily: fontFamilies.mono.join(", "),
  }),
};

// 4. フォントウェイト別のスタイル
export const fontWeightStyles = {
  thin: css({
    fontWeight: fontWeights.thin,
  }),

  extralight: css({
    fontWeight: fontWeights.extralight,
  }),

  regular: css({
    fontWeight: fontWeights.regular,
  }),

  normal: css({
    fontWeight: fontWeights.normal,
  }),

  medium: css({
    fontWeight: fontWeights.medium,
  }),

  semibold: css({
    fontWeight: fontWeights.semibold,
  }),

  bold: css({
    fontWeight: fontWeights.bold,
  }),
};

// 5. 行間（line-height）別のスタイル
export const lineHeightStyles = {
  tight: css({
    lineHeight: lineHeights.tight,
  }),

  normal: css({
    lineHeight: lineHeights.normal,
  }),

  relaxed: css({
    lineHeight: lineHeights.relaxed,
  }),

  loose: css({
    lineHeight: lineHeights.loose,
  }),
};

// 6. 文字間隔（letter-spacing）別のスタイル
export const letterSpacingStyles = {
  tighter: css({
    letterSpacing: letterSpacings.tighter,
  }),

  tight: css({
    letterSpacing: letterSpacings.tight,
  }),

  normal: css({
    letterSpacing: letterSpacings.normal,
  }),

  wide: css({
    letterSpacing: letterSpacings.wide,
  }),

  wider: css({
    letterSpacing: letterSpacings.wider,
  }),

  widest: css({
    letterSpacing: letterSpacings.widest,
  }),
};

// 7. 組み合わせスタイルの例
export const combinedStyles = {
  heroTitle: css({
    fontSize: fluidTypography.display,
    fontWeight: fontWeights.bold,
    lineHeight: lineHeights.tight,
    letterSpacing: letterSpacings.tight,
    fontFamily: fontFamilies.sans.join(", "),
  }),

  cardTitle: css({
    fontSize: fluidTypography.h3,
    fontWeight: fontWeights.semibold,
    lineHeight: lineHeights.tight,
    marginBottom: "0.75rem",
  }),

  cardText: css({
    fontSize: fluidTypography.textBase,
    fontWeight: fontWeights.normal,
    lineHeight: lineHeights.relaxed,
    marginBottom: "1rem",
  }),

  caption: css({
    fontSize: fluidTypography.textSmall,
    fontWeight: fontWeights.medium,
    lineHeight: lineHeights.normal,
    letterSpacing: letterSpacings.wide,
    textTransform: "uppercase",
  }),

  code: css({
    fontSize: fluidTypography.textSmall,
    fontWeight: fontWeights.normal,
    fontFamily: fontFamilies.mono.join(", "),
    backgroundColor: "#f5f5f5",
    padding: "0.25rem 0.5rem",
    borderRadius: "0.25rem",
  }),
};

// 8. レスポンシブタイポグラフィの例
export const responsiveTypographyStyle = css({
  fontSize: fluidTypography.textBase,
  fontWeight: fontWeights.normal,
  lineHeight: lineHeights.normal,

  "@media (min-width: 768px)": {
    fontSize: fluidTypography.textLarge,
    lineHeight: lineHeights.relaxed,
  },

  "@media (min-width: 1024px)": {
    fontSize: fluidTypography.h6,
    lineHeight: lineHeights.normal,
  },
});

/**
 * 動的タイポグラフィ生成関数
 */

// カスタム流体タイプの生成
export const createCustomFluidType = (
  minSize: number,
  maxSize: number,
  weight: FontWeightType = "normal",
  lineHeight: keyof typeof lineHeights = "normal"
) => {
  return css({
    fontSize: `clamp(${minSize}px, ${minSize + (maxSize - minSize) * 0.1}px + ${maxSize - minSize}vw, ${maxSize}px)`,
    fontWeight: getFontWeight(weight),
    lineHeight: lineHeights[lineHeight],
  });
};

// テキストスタイルの組み合わせ生成
export const createTextStyle = (
  size: keyof typeof fluidTypography,
  weight: FontWeightType = "normal",
  lineHeight: keyof typeof lineHeights = "normal",
  letterSpacing: keyof typeof letterSpacings = "normal"
) => {
  return css({
    fontSize: fluidTypography[size],
    fontWeight: getFontWeight(weight),
    lineHeight: lineHeights[lineHeight],
    letterSpacing: letterSpacings[letterSpacing],
  });
};

/**
 * タイポグラフィシステムのガイドライン
 */
export const typographyGuide = {
  /**
   * フォントサイズの使用用途:
   *
   * display (40-56px): ヒーローセクション、メインタイトル
   * h1 (32-40px): ページタイトル、セクション見出し
   * h2 (26-32px): サブセクション見出し
   * h3 (22-28px): カードタイトル、小見出し
   * h4 (20-24px): リスト見出し
   * h5 (18-20px): ラベル、キャプション
   * h6 (16-18px): 小さな見出し
   * textLarge (18-20px): 重要なテキスト
   * textBase (16-18px): 本文テキスト
   * textSmall (12-14px): 補足テキスト、キャプション
   */

  bestPractices: [
    "流体タイプを使用してレスポンシブなフォントサイズを実装",
    "見出しには適切な階層構造を維持",
    "行間は読みやすさを考慮して設定（本文: 1.5-1.75、見出し: 1.2-1.3）",
    "フォントウェイトは視覚的階層を作るために適切に使い分け",
    "文字間隔は大文字や小見出しで調整",
    "フォントファミリーは一貫性を保つ",
  ],

  commonPatterns: [
    "見出し: bold + tight line-height + tight letter-spacing",
    "本文: normal weight + normal/relaxed line-height",
    "キャプション: medium weight + wide letter-spacing + uppercase",
    "コード: mono font + small size + background color",
    "ヒーロータイトル: display size + bold + tight spacing",
  ],
} as const;

/**
 * フォントウェイト情報を取得する関数（デバッグ用）
 */
export const getAllFontWeightInfo = () => {
  return Object.entries(fontWeights).map(([key, value]) => ({
    weight: key as FontWeightType,
    value,
    description: typographyGuide[key as keyof typeof typographyGuide] || "No description available",
  }));
};

// 使用例のエクスポート
export const examples = {
  headingStyles,
  textStyles,
  fontFamilyStyles,
  fontWeightStyles,
  lineHeightStyles,
  letterSpacingStyles,
  combinedStyles,
  responsiveTypographyStyle,
} as const;

// パターン集のエクスポート
export const patterns = {
  combinedStyles,
} as const;

// ユーティリティ関数のエクスポート
export const typographyUtils = {
  createCustomFluidType,
  createTextStyle,
  getAllFontWeightInfo,
} as const;
