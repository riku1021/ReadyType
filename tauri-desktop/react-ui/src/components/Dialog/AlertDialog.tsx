import type { FC } from "react";
import { useId } from "react";
import { useTranslation } from "react-i18next";

import ConfirmAnimation from "@/animations/ConfirmAnimation";
import { Button } from "@/components/Button";

import * as styles from "./Dialog.styles";
import { DialogShell } from "./DialogShell";
import { StatusDialogPanel } from "./StatusDialogPanel";

type AlertDialogProps = {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  text?: string;
  confirmButtonText?: string;
  cancelButtonText?: string;
  onConfirm?: () => void;
};

/**
 * 応答必須の確認ダイアログ（ARIA Alert Dialog / Radix AlertDialog 相当）。
 */
export const AlertDialog: FC<AlertDialogProps> = ({
  isOpen,
  onClose,
  title,
  text,
  confirmButtonText,
  cancelButtonText,
  onConfirm,
}) => {
  const { t } = useTranslation();
  const titleId = useId();
  const confirmText = confirmButtonText ?? t("dialog.confirmDefault");
  const cancelText = cancelButtonText ?? t("dialog.cancelDefault");

  const handleConfirm = () => {
    onConfirm?.();
    onClose();
  };

  const buttonFooter = (
    <div className={styles.buttonContainer}>
      <Button variant="primary" onClick={handleConfirm}>
        {confirmText}
      </Button>
      <Button variant="secondary" onClick={onClose}>
        {cancelText}
      </Button>
    </div>
  );

  return (
    <DialogShell isOpen={isOpen} onClose={onClose} role="alertdialog" ariaLabelledBy={titleId}>
      <StatusDialogPanel
        titleId={titleId}
        title={title}
        text={text}
        media={<ConfirmAnimation />}
        footer={buttonFooter}
      />
    </DialogShell>
  );
};
