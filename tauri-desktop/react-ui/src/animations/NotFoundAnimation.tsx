import { useAtom } from "jotai";
import type { FC } from "react";
import darkAnimation from "@/assets/animations/404_dark.lottie";
import lightAnimation from "@/assets/animations/404_light.lottie";
import { isDarkThemeAtom } from "@/atoms/theme";
import { LottieAnimation } from "@/components/LottieAnimation";
import { useAnimationTransition } from "@/hooks/useAnimationTransition";

type NotFoundAnimationProps = {
  className?: string;
};

const NotFoundAnimation: FC<NotFoundAnimationProps> = ({ className }) => {
  const [isDarkTheme] = useAtom(isDarkThemeAtom);
  const animationPath = isDarkTheme ? darkAnimation : lightAnimation;
  const currentAnimation = useAnimationTransition(animationPath);

  return (
    <div className={className} aria-hidden>
      <LottieAnimation
        src={currentAnimation || animationPath}
        loop={true}
        autoplay={true}
        speed={0.3}
        mode="bounce"
        style={{ width: "100%", height: "100%" }}
      />
    </div>
  );
};

export default NotFoundAnimation;
