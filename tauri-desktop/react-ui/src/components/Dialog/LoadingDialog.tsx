import type { FC } from "react";
import { useId } from "react";

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

export const LoadingDialog: FC<LoadingDialogProps> = ({
  isOpen,
  onClose,
  title = "処理中...",
  text = "しばらくお待ちください",
}) => {
  const titleId = useId();
  const resolvedClose = onClose ?? noopClose;

  return (
    <DialogShell
      isOpen={isOpen}
      onClose={resolvedClose}
      dismissible={false}
      ariaLabelledBy={titleId}
    >
      <StatusDialogPanel
        titleId={titleId}
        title={title}
        text={text}
        media={
          <div className={styles.loadingContainer}>
            <LoadingAnimation />
          </div>
        }
      />
    </DialogShell>
  );
};
