import { cva } from "styled-system/css";

export const keycap = cva({
  base: {
    fontFamily: "body",
    fontWeight: "extrabold",
    color: "ink",
    bg: "white",
  },
  variants: {
    size: {
      sm: {
        borderWidth: "1px",
        borderStyle: "solid",
        borderColor: "line",
        borderBottomWidth: "3px",
        borderRadius: "7px",
        px: "8px",
        fontSize: "0.8125rem",
      },
      mini: {
        borderWidth: "1px",
        borderStyle: "solid",
        borderColor: "line",
        borderBottomWidth: "3px",
        borderRadius: "7px",
        px: "6px",
        fontSize: "0.75rem",
      },
      lg: {
        width: "42px",
        height: "42px",
        display: "grid",
        placeItems: "center",
        fontSize: "1.125rem",
        borderRadius: "12px",
        boxShadow: "0 4px 0 var(--colors-key-shadow)",
      },
    },
  },
  defaultVariants: { size: "sm" },
});
