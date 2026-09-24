import { useEffect, useState } from "react";

const CLOSE_ANIMATION_MS = 300;

type UseModalAnimationOptions = {
  isOpen: boolean;
  onClose: () => void;
  duration?: number;
  autoClose?: boolean;
};

type UseModalAnimationReturn = {
  visible: boolean;
  isClosing: boolean;
};

export const useModalAnimation = ({
  isOpen,
  onClose,
  duration = 2000,
  autoClose = false,
}: UseModalAnimationOptions): UseModalAnimationReturn => {
  const [visible, setVisible] = useState(isOpen);
  const [isClosing, setIsClosing] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setVisible(true);
      setIsClosing(false);
      document.body.classList.add("no-scroll");

      if (autoClose) {
        const timer = window.setTimeout(() => {
          onClose();
        }, duration);
        return () => window.clearTimeout(timer);
      }
      return;
    }

    setIsClosing(true);
    document.body.classList.remove("no-scroll");
    const timer = window.setTimeout(() => {
      setVisible(false);
    }, CLOSE_ANIMATION_MS);
    return () => window.clearTimeout(timer);
  }, [isOpen, onClose, duration, autoClose]);

  useEffect(() => {
    return () => {
      document.body.classList.remove("no-scroll");
    };
  }, []);

  return {
    visible,
    isClosing,
  };
};
