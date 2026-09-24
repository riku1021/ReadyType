import type { FC } from "react";

import { ConfigurableAnimation } from "@/components/LottieAnimation";

type AnimationProps = {
  loop?: boolean;
  autoplay?: boolean;
  speed?: number;
};

const ErrorAnimation: FC<AnimationProps> = (props) => {
  return <ConfigurableAnimation type="error" {...props} />;
};

export default ErrorAnimation;
