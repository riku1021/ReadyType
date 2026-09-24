import { defineConfig } from "@pandacss/dev";

import { conditions, globalCss, themeExtend } from "./src/styles/panda";

export default defineConfig({
  preflight: true,
  include: ["./src/**/*.{js,jsx,ts,tsx}"],
  exclude: ["./node_modules", "./dist"],
  outdir: "src/styled-system",
  minify: true,
  hash: true,
  clean: false,
  jsxFramework: "react",
  globalCss,
  theme: {
    extend: themeExtend,
  },
  conditions,
});
