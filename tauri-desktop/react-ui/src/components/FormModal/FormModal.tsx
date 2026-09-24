import type { FC, ReactNode } from "react";
import { useId } from "react";

import { DialogShell, FormDialogPanel, formShell } from "@/components/Dialog";

type FormModalProps = {
  isOpen: boolean;
  onClose: () => void;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
};

/** フォーム入力用ダイアログ。DialogShell + FormDialogPanel の薄いラッパ。 */
export const FormModal: FC<FormModalProps> = ({ isOpen, onClose, title, children, footer }) => {
  const titleId = useId();

  return (
    <DialogShell isOpen={isOpen} onClose={onClose} ariaLabelledBy={titleId} className={formShell}>
      <FormDialogPanel titleId={titleId} title={title} footer={footer}>
        {children}
      </FormDialogPanel>
    </DialogShell>
  );
};
