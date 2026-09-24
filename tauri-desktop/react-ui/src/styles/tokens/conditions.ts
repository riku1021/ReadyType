export const conditions = {
  light: '[data-theme="light"] &, &.light, &[data-theme="light"]',
  dark: '[data-theme="dark"] &, &.dark, &[data-theme="dark"]',
  focusVisibleWithin: "&:has(:focus-visible)",
} as const;
