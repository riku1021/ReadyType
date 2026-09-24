import checkAnimation from "@/assets/animations/check.lottie";
import confirmAnimation from "@/assets/animations/confirm.lottie";
import errorAnimation from "@/assets/animations/error.lottie";
import loadingDarkAnimation from "@/assets/animations/loading_dark.lottie";
import loadingLightAnimation from "@/assets/animations/loading_light.lottie";

import type { LottieAnimationProps } from "./LottieAnimation";

export type AnimationConfig = {
  src: string;
  defaults: Partial<Omit<LottieAnimationProps, "src">>;
};

export const animationConfigs = {
  check: {
    src: checkAnimation,
    defaults: {
      loop: false,
      autoplay: true,
      speed: 1.75,
    },
  },
  error: {
    src: errorAnimation,
    defaults: {
      loop: false,
      autoplay: true,
      speed: 2,
    },
  },
  confirm: {
    src: confirmAnimation,
    defaults: {
      loop: true,
      autoplay: true,
      speed: 0.6,
    },
  },
  loadingDark: {
    src: loadingDarkAnimation,
    defaults: {
      loop: true,
      autoplay: true,
      speed: 1,
    },
  },
  loadingLight: {
    src: loadingLightAnimation,
    defaults: {
      loop: true,
      autoplay: true,
      speed: 1,
    },
  },
} as const satisfies Record<string, AnimationConfig>;

export type AnimationType = keyof typeof animationConfigs;
