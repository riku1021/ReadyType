import type { FC, MouseEvent, ReactNode, SyntheticEvent } from "react";
import { createContext, useContext, useEffect, useLayoutEffect, useRef } from "react";

import { useModalAnimation } from "@/hooks/useModalAnimation";
import { cx } from "@/styled-system/css";

import * as styles from "./Dialog.styles";

type DialogShellContextValue = {
  isClosing: boolean;
  onClose: () => void;
  dismissible: boolean;
};

const DialogShellContext = createContext<DialogShellContextValue | null>(null);

export function useDialogShell(): DialogShellContextValue {
  const value = useContext(DialogShellContext);
  if (!value) {
    throw new Error("useDialogShell must be used within DialogShell");
  }
  return value;
}

export type DialogShellProps = {
  isOpen: boolean;
  onClose: () => void;
  /** false のとき外側タップ・Escape を無効化（Loading 向け）。閉じる UI は Panel 側で参照 */
  dismissible?: boolean;
  role?: "dialog" | "alertdialog";
  ariaLabelledBy: string;
  className?: string;
  children: ReactNode;
};

/**
 * ネイティブ `<dialog>` + `showModal()` の振る舞い専用 Shell。
 * 見た目・タイトル構造は StatusDialogPanel / FormDialogPanel など子が担う。
 */
export const DialogShell: FC<DialogShellProps> = ({
  isOpen,
  onClose,
  dismissible = true,
  role = "dialog",
  ariaLabelledBy,
  className,
  children,
}) => {
  const dialogRef = useRef<HTMLDialogElement>(null);
  const { visible, isClosing } = useModalAnimation({
    isOpen,
    onClose,
    autoClose: false,
  });

  useLayoutEffect(() => {
    const node = dialogRef.current;
    if (!node || !visible) {
      return;
    }
    if (!node.open) {
      node.showModal();
    }
  }, [visible]);

  useEffect(() => {
    const node = dialogRef.current;
    return () => {
      if (node?.open) {
        node.close();
      }
    };
  }, []);

  if (!visible) {
    return null;
  }

  const handleDialogClick = (event: MouseEvent<HTMLDialogElement>) => {
    if (!dismissible) {
      return;
    }
    if (event.target === event.currentTarget) {
      onClose();
    }
  };

  const handleCancel = (event: SyntheticEvent<HTMLDialogElement>) => {
    event.preventDefault();
    if (dismissible) {
      onClose();
    }
  };

  return (
    <DialogShellContext.Provider value={{ isClosing, onClose, dismissible }}>
      {/* biome-ignore lint/a11y/useKeyWithClickEvents: Escape is handled via onCancel */}
      <dialog
        ref={dialogRef}
        className={cx(styles.shell, className, isClosing ? styles.fadeOut : styles.fadeIn)}
        role={role}
        aria-labelledby={ariaLabelledBy}
        onClick={handleDialogClick}
        onCancel={handleCancel}
      >
        {children}
      </dialog>
    </DialogShellContext.Provider>
  );
};
