import { useEffect, useState } from "react";

import { getTransitionSpeed } from "@/utils/cssUtils";

export const useAnimationTransition = (animationPath: string) => {
  const [currentAnimation, setCurrentAnimation] = useState<string | null>(null);

  useEffect(() => {
    if (currentAnimation === null) {
      setCurrentAnimation(animationPath);
      return;
    }

    if (currentAnimation !== animationPath) {
      // トランジション時間を取得
      const durationInMs = getTransitionSpeed("slow");

      setTimeout(() => {
        setCurrentAnimation(animationPath);
      }, durationInMs);
    }
  }, [animationPath, currentAnimation]);

  return currentAnimation;
};
