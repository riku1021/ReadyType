import { globalCss } from "./bases/global";
import { baseThemeExtend } from "./baseTheme";
import { domainColorTokens } from "./domains";
import { conditions } from "./tokens/conditions";

export { baseThemeExtend } from "./baseTheme";
export { domainColorTokens } from "./domains";
export { conditions, globalCss };

/**
 * `panda.config.ts` の `theme.extend` に渡す値。
 * 汎用 DS（baseThemeExtend）+ アプリ固有色（domainColorTokens）を合成する。
 */
export const themeExtend = {
  ...baseThemeExtend,
  semanticTokens: {
    ...baseThemeExtend.semanticTokens,
    colors: {
      ...baseThemeExtend.semanticTokens.colors,
      ...domainColorTokens,
    },
  },
};
