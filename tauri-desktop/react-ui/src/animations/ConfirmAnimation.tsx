import type { FC } from "react";

import { ConfigurableAnimation } from "@/components/LottieAnimation";

type AnimationProps = {
  loop?: boolean;
  autoplay?: boolean;
  speed?: number;
};

const ConfirmAnimation: FC<AnimationProps> = (props) => {
  return <ConfigurableAnimation type="confirm" {...props} />;
};

export default ConfirmAnimation;
