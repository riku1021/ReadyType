import { defineConfig } from "@pandacss/dev";

import { conditions, globalCss, globalVars, themeExtend } from "./src/styles/panda";

export default defineConfig({
  presets: ["@pandacss/preset-base", "@pandacss/preset-panda"],
  preflight: true,
  include: ["./src/**/*.{js,jsx,ts,tsx,astro}"],
  exclude: ["./styled-system/**", "./node_modules/**", "./dist/**"],
  outdir: "styled-system",
  globalCss,
  globalVars,
  conditions,
  theme: {
    extend: themeExtend,
  },
});
