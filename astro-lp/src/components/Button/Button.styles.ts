import { cva } from "styled-system/css";

export const button = cva({
  base: {
    display: "inline-flex",
    alignItems: "center",
    gap: "8px",
    borderRadius: "pill",
    fontFamily: "body",
    fontWeight: "black",
    borderWidth: "0",
    cursor: "pointer",
    transition: ".2s",
    lineHeight: "1.2",
  },
  variants: {
    variant: {
      main: {
        position: "relative",
        overflow: "hidden",
        bgGradient: "brand",
        color: "white",
        boxShadow: "glow",
        _hover: { transform: "translateY(-3px)", boxShadow: "glowStrong" },
        "&::after": {
          content: '""',
          position: "absolute",
          top: "0",
          left: "-60%",
          width: "40%",
          height: "100%",
          pointerEvents: "none",
          backgroundImage:
            "linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.5), transparent)",
          animation: "sweep 3.2s infinite",
        },
      },
      sub: {
        bg: "white",
        color: "brand",
        boxShadow: "inset 0 0 0 2px var(--colors-line)",
        _hover: { boxShadow: "inset 0 0 0 2px var(--colors-brand)" },
      },
      white: {
        bg: "white",
        color: "brand",
        boxShadow: "white",
        _hover: { transform: "translateY(-3px)" },
      },
    },
    size: {
      md: { px: "32px", py: "16px", fontSize: "1rem" },
      nav: { px: "22px", py: "10px", fontSize: "0.875rem", _navCompact: { display: "none" } },
    },
  },
  defaultVariants: { variant: "main", size: "md" },
});
