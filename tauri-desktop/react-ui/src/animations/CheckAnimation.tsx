import type { FC } from "react";

import { ConfigurableAnimation } from "@/components/LottieAnimation";

type AnimationProps = {
  loop?: boolean;
  autoplay?: boolean;
  speed?: number;
};

const CheckAnimation: FC<AnimationProps> = (props) => {
  return <ConfigurableAnimation type="check" {...props} />;
};

export default CheckAnimation;
