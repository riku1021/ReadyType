import { DotLottieReact } from "@lottiefiles/dotlottie-react";
import type { CSSProperties, FC } from "react";

export type LottieAnimationProps = {
  src: string;
  loop?: boolean;
  autoplay?: boolean;
  speed?: number;
  mode?: "bounce";
  style?: CSSProperties;
  className?: string;
};

export const LottieAnimation: FC<LottieAnimationProps> = ({
  src,
  loop = false,
  autoplay = true,
  speed = 1,
  mode,
  style,
  className,
}) => {
  return (
    <DotLottieReact
      src={src}
      loop={loop}
      autoplay={autoplay}
      speed={speed}
      mode={mode}
      style={style}
      className={className}
    />
  );
};

export default LottieAnimation;
