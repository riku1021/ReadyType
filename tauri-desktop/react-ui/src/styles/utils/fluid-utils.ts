// 流体タイプ計算のユーティリティ関数
// vanilla-extractファイルではないため、関数をエクスポート可能

/**
 * 流体タイプ（レスポンシブフォントサイズ）を計算する関数
 * @param minFont 最小フォントサイズ（px）
 * @param maxFont 最大フォントサイズ（px）
 * @param minVw 最小ビューポート幅（px、デフォルト: 320）
 * @param maxVw 最大ビューポート幅（px、デフォルト: 1200）
 * @returns clamp()を使用したCSSの流体タイプ値
 */
export const fluidType = (
  minFont: number,
  maxFont: number,
  minVw: number = 320,
  maxVw: number = 1200
): string => {
  const slope = (maxFont - minFont) / (maxVw - minVw);
  const intercept = minFont - slope * minVw;
  const slopeVw = slope * 100;

  return `clamp(${minFont}px, ${intercept}px + ${slopeVw}vw, ${maxFont}px)`;
};

/**
 * 流体スペーシングを計算する関数
 * @param minSpace 最小スペース（rem）
 * @param maxSpace 最大スペース（rem）
 * @param minVw 最小ビューポート幅（px、デフォルト: 320）
 * @param maxVw 最大ビューポート幅（px、デフォルト: 1200）
 * @returns clamp()を使用したCSSの流体スペーシング値
 */
export const fluidSpacing = (
  minSpace: number,
  maxSpace: number,
  minVw: number = 320,
  maxVw: number = 1200
): string => {
  const slope = (maxSpace - minSpace) / (maxVw - minVw);
  const intercept = minSpace - slope * minVw;
  const slopeVw = slope * 100;

  return `clamp(${minSpace}rem, ${intercept}rem + ${slopeVw}vw, ${maxSpace}rem)`;
};

/**
 * pxをremに変換する関数
 * @param px ピクセル値
 * @param baseFontSize ベースフォントサイズ（デフォルト: 16）
 * @returns rem値の文字列
 */
export const pxToRem = (px: number, baseFontSize: number = 16): string => {
  return `${px / baseFontSize}rem`;
};

/**
 * remをpxに変換する関数
 * @param rem rem値
 * @param baseFontSize ベースフォントサイズ（デフォルト: 16）
 * @returns px値
 */
export const remToPx = (rem: number, baseFontSize: number = 16): number => {
  return rem * baseFontSize;
};

// よく使用する流体タイプのプリセット生成
export const createFluidPresets = () => ({
  // テキストサイズ
  textXs: fluidType(12, 14),
  textSm: fluidType(14, 16),
  textBase: fluidType(16, 18),
  textLg: fluidType(18, 20),
  textXl: fluidType(20, 24),

  // 見出し
  h6: fluidType(16, 18),
  h5: fluidType(18, 20),
  h4: fluidType(20, 24),
  h3: fluidType(22, 28),
  h2: fluidType(26, 32),
  h1: fluidType(32, 40),

  // 大きな見出し
  display: fluidType(40, 56),
});

// 型定義
export type FluidPreset = keyof ReturnType<typeof createFluidPresets>;
