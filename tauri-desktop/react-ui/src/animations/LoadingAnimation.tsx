import { useAtomValue } from "jotai";
import type { FC } from "react";
import { isDarkThemeAtom } from "@/atoms/theme";
import { ConfigurableAnimation } from "@/components/LottieAnimation";

const LoadingAnimation: FC = () => {
  const isDarkTheme = useAtomValue(isDarkThemeAtom);
  const animationType = isDarkTheme ? "loadingDark" : "loadingLight";

  return <ConfigurableAnimation type={animationType} />;
};

export default LoadingAnimation;
