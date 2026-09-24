/**
 * スペーシング使用例
 * 8pxベースのスペーシングシステムの使用方法とベストプラクティスを示す
 */

import { css } from "@/styled-system/css";

import { type SpacingLayerType, spacingLayers } from "../tokens/spacing";

/**
 * スペーシング使用例のスタイル定義
 */

// 1. 基本的なマージン・パディングの使用例
export const cardStyle = css({
  padding: `${spacingLayers.md}px`, // 24px
  margin: `${spacingLayers.sm}px`, // 16px
  backgroundColor: "white",
  borderRadius: "8px",
  boxShadow: "0 2px 4px rgba(0, 0, 0, 0.1)",
});

/**
 * 動的なスペーシング管理の例
 */

// スペーシング値を取得するヘルパー関数
export const getSpacing = (key: SpacingLayerType): string => {
  return `${spacingLayers[key]}px`;
};

// 複数のスペーシング値を組み合わせる関数
export const combineSpacing = (...keys: SpacingLayerType[]): string => {
  return keys.map((key) => getSpacing(key)).join(" ");
};

// 使用例: 複数方向のパディング
export const complexPaddingStyle = css({
  padding: combineSpacing("sm", "md"), // "16px 24px"
});

/**
 * コンポーネント別のスペーシングパターン
 */

// カードコンポーネントのスペーシングパターン
export const cardPatterns = {
  compact: css({
    padding: getSpacing("xs"), // 8px
    margin: getSpacing("xs"), // 8px
  }),

  normal: css({
    padding: getSpacing("sm"), // 16px
    margin: getSpacing("sm"), // 16px
  }),

  spacious: css({
    padding: getSpacing("lg"), // 32px
    margin: getSpacing("md"), // 24px
  }),
};

// モーダルのスペーシングパターン
export const modalPatterns = {
  overlay: css({
    padding: getSpacing("xl"), // 48px
  }),

  content: css({
    padding: getSpacing("lg"), // 32px
    margin: getSpacing("md"), // 24px
  }),

  actions: css({
    marginTop: getSpacing("lg"), // 32px
    gap: getSpacing("sm"), // 16px
    display: "flex",
    justifyContent: "flex-end",
  }),
};

// 使用例のエクスポート
export const examples = {
  cardStyle,
  complexPaddingStyle,
} as const;

// パターン集のエクスポート
export const patterns = {
  cardPatterns,
  modalPatterns,
} as const;

// ユーティリティ関数のエクスポート
export const spacingUtils = {
  getSpacing,
  combineSpacing,
} as const;
