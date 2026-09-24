export const keyframes = {
  fadeInOverlay: {
    from: { opacity: "0" },
    to: { opacity: "1" },
  },
  fadeOutOverlay: {
    from: { opacity: "1" },
    to: { opacity: "0" },
  },
  fadeInContent: {
    from: { opacity: "0", transform: "scale(0.9)" },
    to: { opacity: "1", transform: "scale(1)" },
  },
  fadeOutContent: {
    from: { opacity: "1", transform: "scale(1)" },
    to: { opacity: "0", transform: "scale(0.9)" },
  },
  alertProgressBar: {
    from: { transform: "scaleX(1)" },
    to: { transform: "scaleX(0)" },
  },
  slideInFromRight: {
    from: { opacity: "0", transform: "translateX(0.75rem)" },
    to: { opacity: "1", transform: "translateX(0)" },
  },
  slideInFromLeft: {
    from: { opacity: "0", transform: "translateX(-0.75rem)" },
    to: { opacity: "1", transform: "translateX(0)" },
  },
} as const;
