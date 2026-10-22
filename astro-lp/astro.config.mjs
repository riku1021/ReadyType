// @ts-check

import { fileURLToPath } from "node:url";
import pandacss from "@pandacss/vite";
import { defineConfig } from "astro/config";

const styledSystem = fileURLToPath(new URL("./styled-system", import.meta.url));

// https://astro.build/config
export default defineConfig({
  vite: {
    plugins: [pandacss()],
    resolve: {
      alias: {
        "styled-system": styledSystem,
      },
    },
  },
});
