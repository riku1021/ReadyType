import type { FC } from "react";
import { useEffect, useId } from "react";

import CheckAnimation from "@/animations/CheckAnimation";
import ErrorAnimation from "@/animations/ErrorAnimation";

import * as styles from "./Dialog.styles";
import { DialogShell } from "./DialogShell";
import { StatusDialogPanel } from "./StatusDialogPanel";

export type StatusDialogVariant = "success" | "error";

type StatusDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  text?: string;
  duration?: number;
  variant: StatusDialogVariant;
};

const DEFAULT_DURATION: Record<StatusDialogVariant, number> = {
  success: 2000,
  error: 3000,
};

/**
 * 自動クローズするステータス系モーダル。
 * Feedback としては Toast が一般的だが、本アプリでは意図的に modal dialog で表示する。
 */
export const StatusDialog: FC<StatusDialogProps> = ({
  isOpen,
  onClose,
  title,
  text,
  duration,
  variant,
}) => {
  const titleId = useId();
  const resolvedDuration = duration ?? DEFAULT_DURATION[variant];

  useEffect(() => {
    if (!isOpen) {
      return;
    }
    const timer = window.setTimeout(() => {
      onClose();
    }, resolvedDuration);
    return () => window.clearTimeout(timer);
  }, [isOpen, onClose, resolvedDuration]);

  const progressBarFooter = (
    <div className={styles.progressBarContainer}>
      <div
        className={styles.progressBarStyle}
        style={{ animationDuration: `${resolvedDuration}ms` }}
      />
    </div>
  );

  return (
    <DialogShell isOpen={isOpen} onClose={onClose} ariaLabelledBy={titleId}>
      <StatusDialogPanel
        titleId={titleId}
        title={title}
        text={text}
        media={variant === "success" ? <CheckAnimation /> : <ErrorAnimation />}
        footer={progressBarFooter}
      />
    </DialogShell>
  );
};

type StatusDialogShortcutProps = Omit<StatusDialogProps, "variant">;

/** @see StatusDialog — variant="success" の短縮形 */
export const SuccessDialog: FC<StatusDialogShortcutProps> = (props) => (
  <StatusDialog {...props} variant="success" />
);

/** @see StatusDialog — variant="error" の短縮形 */
export const ErrorDialog: FC<StatusDialogShortcutProps> = (props) => (
  <StatusDialog {...props} variant="error" />
);
