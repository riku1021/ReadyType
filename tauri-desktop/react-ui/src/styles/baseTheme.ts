import { colorTokens } from "./tokens/color";
import { keyframes } from "./tokens/keyframes";
import { breakpointTokens } from "./tokens/responsive";
import { spacingTokens } from "./tokens/spacing";
import { pandaTypographyTokens } from "./tokens/typography";
import { zIndexTokens } from "./tokens/zindex";

/**
 * 汎用デザインシステムのみ（tokens/）。
 * 別プロジェクトへ切り出すときはこのオブジェクトをベースにする。
 * アプリ固有色は `domains/` 側で合成する。
 */
export const baseThemeExtend = {
  breakpoints: breakpointTokens,
  keyframes,
  semanticTokens: {
    colors: colorTokens,
  },
  tokens: {
    ...pandaTypographyTokens,
    zIndex: zIndexTokens,
    spacing: spacingTokens,
  },
};
