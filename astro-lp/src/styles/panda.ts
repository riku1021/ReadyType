import { globalCss } from "./bases/global";
import { conditions } from "./conditions";
import { keyframes, tokens } from "./tokens";

export { conditions, globalCss };

export const globalVars = {
  "--ang": {
    syntax: "<angle>" as const,
    inherits: false,
    initialValue: "0deg",
  },
};

export const themeExtend = {
  tokens,
  keyframes,
  colorPalette: { enabled: false },
};
