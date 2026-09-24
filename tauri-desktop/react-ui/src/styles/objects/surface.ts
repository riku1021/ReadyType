import { css } from "@/styled-system/css";

/**
 * 面（surface）コンテキスト。
 * 背景色と一緒に、子孫 Button が継承する影・枠幅用 CSS 変数をセットする。
 *
 * - main: background.main 上 → shadow.primary、枠付き Button は 1px
 * - sub: background.sub 上 → shadow.secondary、枠付き Button は 2px
 *
 * 利用: `css(surfaceSub, { padding: "sm", ... })`
 * Button 自身の塗りには使わない（親面の変数を上書きしてしまうため）。
 */
export const surfaceMain = css.raw({
  backgroundColor: "background.main",
  "--button-surface-shadow": "{colors.shadow.primary}",
  "--button-framed-border-width": "1px",
  "--button-primary-border-width": "1px",
});

export const surfaceSub = css.raw({
  backgroundColor: "background.sub",
  "--button-surface-shadow": "{colors.shadow.secondary}",
  "--button-framed-border-width": "2px",
  "--button-primary-border-width": "0px",
});

/** 背景は別指定で、コンテキスト変数だけ載せるとき用 */
export const surfaceMainContext = css.raw({
  "--button-surface-shadow": "{colors.shadow.primary}",
  "--button-framed-border-width": "1px",
  "--button-primary-border-width": "1px",
});

export const surfaceSubContext = css.raw({
  "--button-surface-shadow": "{colors.shadow.secondary}",
  "--button-framed-border-width": "2px",
  "--button-primary-border-width": "0px",
});
