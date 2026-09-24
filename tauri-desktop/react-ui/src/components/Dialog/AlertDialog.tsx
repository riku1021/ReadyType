import type { FC } from "react";
import { useId } from "react";

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
  confirmButtonText = "はい",
  cancelButtonText = "キャンセル",
  onConfirm,
}) => {
  const titleId = useId();

  const handleConfirm = () => {
    onConfirm?.();
    onClose();
  };

  const buttonFooter = (
    <div className={styles.buttonContainer}>
      <Button variant="primary" onClick={handleConfirm}>
        {confirmButtonText}
      </Button>
      <Button variant="secondary" onClick={onClose}>
        {cancelButtonText}
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
