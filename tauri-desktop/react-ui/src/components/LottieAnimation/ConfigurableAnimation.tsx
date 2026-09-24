import type { FC } from "react";

import { type AnimationType, animationConfigs } from "./animationConfigs";
import LottieAnimation, { type LottieAnimationProps } from "./LottieAnimation";

export type ConfigurableAnimationProps = Omit<LottieAnimationProps, "src"> & {
  type: AnimationType;
};

export const ConfigurableAnimation: FC<ConfigurableAnimationProps> = ({ type, ...props }) => {
  const config = animationConfigs[type];

  return <LottieAnimation {...config.defaults} {...props} src={config.src} />;
};

export default ConfigurableAnimation;
