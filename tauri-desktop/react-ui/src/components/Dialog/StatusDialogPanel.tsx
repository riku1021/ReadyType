import type { FC, ReactNode } from "react";
import { IoIosClose } from "react-icons/io";

import { cx } from "@/styled-system/css";

import * as styles from "./Dialog.styles";
import { useDialogShell } from "./DialogShell";

type StatusDialogPanelProps = {
  titleId: string;
  title: string;
  text?: string;
  media?: ReactNode;
  footer?: ReactNode;
};

/** Status / Alert / Loading 向けの中央寄せパネル */
export const StatusDialogPanel: FC<StatusDialogPanelProps> = ({
  titleId,
  title,
  text,
  media,
  footer,
}) => {
  const { isClosing, onClose, dismissible } = useDialogShell();

  return (
    <div
      className={cx(styles.statusPanel, isClosing ? styles.contentFadeOut : styles.contentFadeIn)}
    >
      {dismissible ? (
        <button type="button" className={styles.closeIcon} onClick={onClose} aria-label="閉じる">
          <IoIosClose />
        </button>
      ) : null}

      {media ? <div className={styles.animationContainer}>{media}</div> : null}

      <h2 id={titleId} className={styles.statusTitle}>
        {title}
      </h2>

      {text ? <div className={styles.statusBody}>{text}</div> : null}

      {footer ? <div className={styles.statusFooter}>{footer}</div> : null}
    </div>
  );
};
