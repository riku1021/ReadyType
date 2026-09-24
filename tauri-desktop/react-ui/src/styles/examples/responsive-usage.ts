/**
 * レスポンシブデザイン使用例
 * ブレイクポイントとメディアクエリの使用方法とベストプラクティスを示す
 */

import { css } from "@/styled-system/css";

import { type BreakpointType, breakpoints } from "../tokens/responsive";

/**
 * レスポンシブデザイン使用例のスタイル定義
 */

// 1. 基本的なモバイルファーストアプローチ
export const mobileFirstContainerStyle = css({
  padding: "16px", // モバイル: 16px
  fontSize: "14px", // モバイル: 14px

  "@media (min-width: 640px)": {
    padding: "24px", // タブレット以上: 24px
    fontSize: "16px", // タブレット以上: 16px
  },

  "@media (min-width: 1024px)": {
    padding: "32px", // デスクトップ以上: 32px
    fontSize: "18px", // デスクトップ以上: 18px
  },
});

// 2. デスクトップファーストアプローチ
export const desktopFirstContainerStyle = css({
  padding: "32px", // デスクトップ: 32px
  fontSize: "18px", // デスクトップ: 18px

  "@media (max-width: 1023px)": {
    padding: "24px", // タブレット以下: 24px
    fontSize: "16px", // タブレット以下: 16px
  },

  "@media (max-width: 639px)": {
    padding: "16px", // モバイル以下: 16px
    fontSize: "14px", // モバイル以下: 14px
  },
});

// 3. 特定の範囲でのスタイル適用
export const tabletOnlyStyle = css({
  display: "block",

  "@media (min-width: 768px) and (max-width: 1023px)": {
    display: "flex", // タブレット範囲(768px~1023px)のみフレックス
    justifyContent: "space-between",
  },
});

// 4. グリッドレイアウトのレスポンシブ対応
export const responsiveGridStyle = css({
  display: "grid",
  gridTemplateColumns: "1fr", // モバイル: 1列
  gap: "16px",

  "@media (min-width: 640px)": {
    gridTemplateColumns: "repeat(2, 1fr)", // タブレット: 2列
    gap: "20px",
  },

  "@media (min-width: 768px)": {
    gridTemplateColumns: "repeat(3, 1fr)", // デスクトップ: 3列
    gap: "24px",
  },

  "@media (min-width: 1280px)": {
    gridTemplateColumns: "repeat(4, 1fr)", // 大画面: 4列
    gap: "32px",
  },
});

// 5. ナビゲーションのレスポンシブ対応
export const responsiveNavigationStyle = css({
  display: "flex",
  flexDirection: "column", // モバイル: 縦並び
  gap: "8px",

  "@media (min-width: 768px)": {
    flexDirection: "row", // タブレット以上: 横並び
    gap: "24px",
    alignItems: "center",
  },
});

// 6. フォントサイズのレスポンシブ対応
export const responsiveHeadingStyle = css({
  fontSize: "24px", // モバイル
  lineHeight: "1.2",

  "@media (min-width: 640px)": {
    fontSize: "32px", // タブレット
  },

  "@media (min-width: 1024px)": {
    fontSize: "48px", // デスクトップ
  },

  "@media (min-width: 1280px)": {
    fontSize: "56px", // 大画面
  },
});

// 7. 画像のレスポンシブ対応
export const responsiveImageContainerStyle = css({
  width: "100%",
  height: "200px", // モバイル
  overflow: "hidden",
  borderRadius: "8px",

  "@media (min-width: 640px)": {
    height: "250px", // タブレット
  },

  "@media (min-width: 1024px)": {
    height: "300px", // デスクトップ
  },

  "& img": {
    width: "100%",
    height: "100%",
    objectFit: "cover",
  },
});

// 8. ボタンのレスポンシブ対応
export const responsiveButtonStyle = css({
  padding: "12px 16px", // モバイル
  fontSize: "14px",
  width: "100%", // モバイルでは全幅

  "@media (min-width: 640px)": {
    width: "auto", // タブレット以上では自動幅
    padding: "12px 24px",
    fontSize: "16px",
  },
});

/**
 * レスポンシブデザインのガイドライン
 */
export const responsiveGuide = {
  /**
   * ブレイクポイントの使用用途:
   *
   * xs (0px~639px): スマートフォン
   * sm (640px~767px): 大きなスマートフォン、小さなタブレット
   * md (768px~1023px): タブレット
   * lg (1024px~1279px): 小さなデスクトップ、ラップトップ
   * xl (1280px~): 大きなデスクトップ
   */

  bestPractices: [
    "モバイルファーストアプローチを基本とする",
    "コンテンツの階層を画面サイズに応じて調整",
    "タッチデバイスでは十分なタップ領域を確保",
    "テキストの可読性を全デバイスで保つ",
    "画像は適切なサイズで配信し、パフォーマンスを考慮",
    "ナビゲーションは画面サイズに応じて形式を変更",
  ],

  commonPatterns: [
    "モバイル: 縦並び、タブレット以上: 横並び",
    "モバイル: 全幅ボタン、デスクトップ: 自動幅ボタン",
    "モバイル: ハンバーガーメニュー、デスクトップ: 横並びナビ",
    "モバイル: 1列グリッド、タブレット: 2列、デスクトップ: 3-4列",
    "モバイル: スタックレイアウト、デスクトップ: サイドバーレイアウト",
  ],
} as const;

// 使用例のエクスポート
export const examples = {
  mobileFirstContainerStyle,
  desktopFirstContainerStyle,
  tabletOnlyStyle,
  responsiveGridStyle,
  responsiveNavigationStyle,
  responsiveHeadingStyle,
  responsiveImageContainerStyle,
  responsiveButtonStyle,
} as const;

/**
 * 動的メディアクエリの使用例（関数として提供）
 */
export const createResponsiveStyle = (
  minBreakpoint?: BreakpointType,
  maxBreakpoint?: BreakpointType
) => {
  const baseStyle = {
    padding: "16px",
    fontSize: "14px",
  };

  if (minBreakpoint && maxBreakpoint) {
    const minValue = breakpoints[minBreakpoint];
    const maxValue = breakpoints[maxBreakpoint] - 1;
    return {
      ...baseStyle,
      [`@media (min-width: ${minValue}px) and (max-width: ${maxValue}px)`]: {
        padding: "24px",
        fontSize: "16px",
      },
    };
  } else if (minBreakpoint) {
    const minValue = breakpoints[minBreakpoint];
    return {
      ...baseStyle,
      [`@media (min-width: ${minValue}px)`]: {
        padding: "24px",
        fontSize: "16px",
      },
    };
  }

  return baseStyle;
};
