// =============================================================================
// レスポンシブユーティリティ
// =============================================================================

// ブレイクポイント定義
export const breakpoints = {
  sm: 640,
  md: 768,
  lg: 1024,
  xl: 1280,
} as const;

export type BreakpointType = keyof typeof breakpoints;

/** Panda `theme.breakpoints` 用（px 文字列） */
export const breakpointTokens = {
  sm: `${breakpoints.sm}px`,
  md: `${breakpoints.md}px`,
  lg: `${breakpoints.lg}px`,
  xl: `${breakpoints.xl}px`,
} as const;

// =============================================================================
// メディアクエリ生成関数
// =============================================================================

/**
 * 指定したブレイクポイント以上の画面幅に適用するメディアクエリを生成
 * モバイルファーストアプローチ用
 * @param breakpoint ブレイクポイント名
 * @returns メディアクエリ文字列
 */
export const respondTo = (breakpoint: BreakpointType): string => {
  return `screen and (min-width: ${breakpoints[breakpoint]}px)`;
};

/**
 * 指定したブレイクポイント未満の画面幅に適用するメディアクエリを生成
 * デスクトップファーストアプローチ用
 * @param breakpoint ブレイクポイント名
 * @returns メディアクエリ文字列
 */
export const respondToMax = (breakpoint: BreakpointType): string => {
  return `screen and (max-width: ${breakpoints[breakpoint] - 1}px)`;
};

/**
 * 指定した2つのブレイクポイント間の画面幅に適用するメディアクエリを生成
 * @param min 最小ブレイクポイント名
 * @param max 最大ブレイクポイント名
 * @returns メディアクエリ文字列
 */
export const respondBetween = (min: BreakpointType, max: BreakpointType): string => {
  return `screen and (min-width: ${breakpoints[min]}px) and (max-width: ${breakpoints[max] - 1}px)`;
};
