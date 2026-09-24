import type { FC, ReactNode } from "react";
import { IoIosClose } from "react-icons/io";

import { cx } from "@/styled-system/css";

import * as styles from "./Dialog.styles";
import { useDialogShell } from "./DialogShell";

type FormDialogPanelProps = {
  titleId: string;
  title: string;
  children: ReactNode;
  footer?: ReactNode;
};

/** フォーム入力向けの左寄せパネル */
export const FormDialogPanel: FC<FormDialogPanelProps> = ({ titleId, title, children, footer }) => {
  const { isClosing, onClose, dismissible } = useDialogShell();

  return (
    <div className={cx(styles.formPanel, isClosing ? styles.contentFadeOut : styles.contentFadeIn)}>
      {dismissible ? (
        <button type="button" className={styles.closeIcon} onClick={onClose} aria-label="閉じる">
          <IoIosClose />
        </button>
      ) : null}

      <h2 id={titleId} className={styles.formTitle}>
        {title}
      </h2>

      <div className={styles.formBody}>{children}</div>

      {footer ? <div className={styles.formFooter}>{footer}</div> : null}
    </div>
  );
};
