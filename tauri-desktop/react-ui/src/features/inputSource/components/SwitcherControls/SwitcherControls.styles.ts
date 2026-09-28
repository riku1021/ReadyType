import { css, cva } from "@/styled-system/css";

export const iconButton = css({
  position: "relative",
  width: "1.75rem",
  height: "1.75rem",
  background: "transparent",
  border: "none",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: "0",
  color: "icon.secondary",
  _hover: {
    color: "text.main",
  },
  _disabled: {
    opacity: 0.4,
    cursor: "not-allowed",
  },
});

export const icon = css({
  width: "1.75rem",
  height: "1.75rem",
});

/** ヒット領域は ThemeToggle / 電源と同じ 1.75rem */
export const playPause = css({
  position: "relative",
  width: "1.75rem",
  height: "1.75rem",
  background: "transparent",
  border: "none",
  outline: "none",
  cursor: "pointer",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  padding: "0",
  color: "icon.secondary",
  WebkitTapHighlightColor: "transparent",
  _hover: {
    color: "text.main",
  },
  _disabled: {
    opacity: 0.4,
    cursor: "not-allowed",
  },
});

export const playPauseDisabled = css({
  opacity: 0.4,
  cursor: "not-allowed",
});

/** アイコン見た目の上に重ねる透明ヒット領域（select / resume button 共用） */
export const hitTarget = css({
  position: "absolute",
  inset: "0",
  width: "100%",
  height: "100%",
  opacity: "0",
  cursor: "pointer",
  appearance: "none",
  border: "none",
  background: "transparent",
  padding: "0",
  fontSize: "1rem",
  _disabled: {
    cursor: "not-allowed",
  },
});

/** HamzaHerbou / YouTube 風。Md アイコンの視覚サイズに合わせて縮小 */
export const playPauseGlyph = cva({
  base: {
    position: "relative",
    width: "0.95rem",
    height: "1.15rem",
    flexShrink: "0",
    transitionProperty: "clip-path",
    transitionDuration: "150ms",
    transitionTimingFunction: "ease-out",
    clipPath: "polygon(0 0, 0 100%, 100% 100%, 100% 0)",
  },
  variants: {
    playing: {
      true: {
        clipPath: "polygon(0 0, 0 100%, 100% 50%, 100% 50%)",
      },
      false: {},
    },
  },
  defaultVariants: {
    playing: false,
  },
});

export const playPauseBar = cva({
  base: {
    position: "absolute",
    top: "0",
    width: "30%",
    height: "100%",
    backgroundColor: "currentColor",
    transitionProperty: "transform",
    transitionDuration: "300ms",
    transitionTimingFunction: "ease-out",
  },
  variants: {
    side: {
      start: {
        left: "0",
        transformOrigin: "left",
      },
      end: {
        right: "0",
        transformOrigin: "right",
      },
    },
    playing: {
      true: {
        transform: "scaleX(2)",
      },
      false: {
        transform: "scaleX(1)",
      },
    },
  },
  defaultVariants: {
    playing: false,
  },
});
