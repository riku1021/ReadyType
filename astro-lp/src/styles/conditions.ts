import { defineConditions } from "@pandacss/dev";

export const conditions = defineConditions({
  extend: {
    bp700: "@media (max-width: 700px)",
    bp900: "@media (max-width: 900px)",
    bp1100: "@media (max-width: 1100px)",
  },
});
