/**
 * 定義済みのトランジション時間を取得する関数
 * CSS変数から値を取得し、秒からミリ秒に変換します
 * @param speed - トランジションスピード指定
 * @returns トランジション時間（ミリ秒）
 */
export const getTransitionSpeed = (
  speed: "fast" | "normal" | "slow" | "slower" = "normal"
): number => {
  const cssVarMap = {
    fast: "--transition-duration-fast",
    normal: "--transition-duration-normal",
    slow: "--transition-duration-slow",
    slower: "--transition-duration-slower",
  };
  const root = document.documentElement;
  const transitionDuration = getComputedStyle(root).getPropertyValue(cssVarMap[speed]);

  // 秒をミリ秒に変換（例: "0.5s" → 500）
  return parseFloat(transitionDuration) * 1000;
};
