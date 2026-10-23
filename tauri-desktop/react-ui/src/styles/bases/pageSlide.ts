/**
 * 一覧 → 詳細の水平スライド。`pageSlide("forward" | "back")` が type を渡したときだけ動く。
 * 方向は Flutter テンプレートの stack push（新しい画面が右から入る）と pop（前の画面が左から入る）。
 * 時間は main shell の PageView と同じ 300ms / ease-in-out。
 *
 * `viewTransitionName` は Panda の静的抽出のため、Header / rootLayout の `css()` に同じ文字列を直書きする。
 */

const pageSlideHeaderName = "page-slide-header";
const pageSlideMainName = "page-slide-main";
export const pageSlideForwardType = "slide-right";
export const pageSlideBackType = "slide-left";

const pageSlideDuration = "300ms";

const pageSlideNames = [pageSlideHeaderName, pageSlideMainName] as const;

function slideSelectors(type: string, pseudo: "old" | "new" | "group"): string {
  return pageSlideNames
    .map((name) => `html:active-view-transition-type(${type})::view-transition-${pseudo}(${name})`)
    .join(", ");
}

function rootSelectors(type: string): string {
  return [
    `html:active-view-transition-type(${type})::view-transition-old(root)`,
    `html:active-view-transition-type(${type})::view-transition-new(root)`,
  ].join(", ");
}

function slide(animationName: string) {
  return {
    animationName,
    animationDuration: pageSlideDuration,
    animationTimingFunction: "ease-in-out",
    animationFillMode: "both",
    mixBlendMode: "normal",
  };
}

export const pageSlideGlobalCss = {
  [`${slideSelectors(pageSlideForwardType, "group")}, ${slideSelectors(pageSlideBackType, "group")}`]:
    {
      overflow: "clip",
      backgroundColor: "background.main",
    },
  [slideSelectors(pageSlideForwardType, "old")]: slide("slidePageOutToLeft"),
  [slideSelectors(pageSlideForwardType, "new")]: slide("slidePageInFromRight"),
  [slideSelectors(pageSlideBackType, "old")]: slide("slidePageOutToRight"),
  [slideSelectors(pageSlideBackType, "new")]: slide("slidePageInFromLeft"),
  [`${rootSelectors(pageSlideForwardType)}, ${rootSelectors(pageSlideBackType)}`]: {
    animation: "none",
    mixBlendMode: "normal",
  },
};
