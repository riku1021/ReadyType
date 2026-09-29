import type { FC } from "react";
import { useId } from "react";
import { useTranslation } from "react-i18next";

import LoadingAnimation from "@/animations/LoadingAnimation";

import * as styles from "./Dialog.styles";
import { DialogShell } from "./DialogShell";
import { StatusDialogPanel } from "./StatusDialogPanel";

type LoadingDialogProps = {
  isOpen: boolean;
  onClose?: () => void;
  title?: string;
  text?: string;
};

const noopClose = () => {
  return;
};

export const LoadingDialog: FC<LoadingDialogProps> = ({ isOpen, onClose, title, text }) => {
  const { t } = useTranslation();
  const titleId = useId();
  const resolvedClose = onClose ?? noopClose;
  const resolvedTitle = title ?? t("dialog.loadingTitleDefault");
  const resolvedText = text ?? t("dialog.loadingTextDefault");

  return (
    <DialogShell
      isOpen={isOpen}
      onClose={resolvedClose}
      dismissible={false}
      ariaLabelledBy={titleId}
    >
      <StatusDialogPanel
        titleId={titleId}
        title={resolvedTitle}
        text={resolvedText}
        media={
          <div className={styles.loadingContainer}>
            <LoadingAnimation />
          </div>
        }
      />
    </DialogShell>
  );
};
