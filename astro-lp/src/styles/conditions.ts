import { defineConditions } from "@pandacss/dev";

export const conditions = defineConditions({
  extend: {
    menu: "@media (max-width: 860px)",
    navCompact: "@media (max-width: 440px)",
    short: "@media (max-height: 720px)",
    squat: "@media (max-height: 600px)",
    wide: "@media (min-width: 1200px)",
    stageTight: "@container (max-width: 470px)",
    glanceTight: "@container (max-width: 680px)",
    frameTight: "@container (max-width: 560px)",
    howTight: "@container (max-width: 860px)",
    bandTight: "@container (max-width: 700px)",
  },
});
