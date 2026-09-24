/**
 * スペーシングトークン（8px ベース）
 * Panda へは `spacingTokens` を登録する。
 */

const SPACING_UNIT = 8;

export const spacingLayers = {
  xs: SPACING_UNIT,
  sm: SPACING_UNIT * 2,
  md: SPACING_UNIT * 3,
  lg: SPACING_UNIT * 4,
  xl: SPACING_UNIT * 6,
} as const;

export type SpacingLayerType = keyof typeof spacingLayers;

export const spacingTokens = {
  xs: { value: `${spacingLayers.xs}px` },
  sm: { value: `${spacingLayers.sm}px` },
  md: { value: `${spacingLayers.md}px` },
  lg: { value: `${spacingLayers.lg}px` },
  xl: { value: `${spacingLayers.xl}px` },
} as const;
