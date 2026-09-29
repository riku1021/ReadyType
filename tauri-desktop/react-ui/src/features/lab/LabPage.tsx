import { useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/Button";
import { AlertDialog, ErrorDialog, LoadingDialog, SuccessDialog } from "@/components/Dialog";
import { FormModal } from "@/components/FormModal";

import * as styles from "./LabPage.styles";

export function LabPage() {
  const { t } = useTranslation();
  const [infoResult, setInfoResult] = useState("");
  const [isSuccessDialogOpen, setIsSuccessDialogOpen] = useState(false);
  const [isErrorDialogOpen, setIsErrorDialogOpen] = useState(false);
  const [isAlertDialogOpen, setIsAlertDialogOpen] = useState(false);
  const [isLoadingDialogOpen, setIsLoadingDialogOpen] = useState(false);
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);

  const handleLoadingDialog = () => {
    setIsLoadingDialogOpen(true);
    window.setTimeout(() => {
      setIsLoadingDialogOpen(false);
    }, 3000);
  };

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>{t("lab.heading")}</h1>
      <p className={styles.lead}>{t("lab.lead")}</p>

      <h2 className={styles.sectionTitle}>{t("lab.sectionDialogs")}</h2>
      <div className={styles.buttonGroup}>
        <Button variant="primary" onClick={() => setIsSuccessDialogOpen(true)}>
          {t("lab.buttonSuccess")}
        </Button>
        <Button variant="primary" onClick={() => setIsErrorDialogOpen(true)}>
          {t("lab.buttonError")}
        </Button>
        <Button variant="secondary" onClick={() => setIsAlertDialogOpen(true)}>
          {t("lab.buttonConfirm")}
        </Button>
        <Button variant="primary" onClick={handleLoadingDialog}>
          {t("lab.buttonLoading")}
        </Button>
        <Button variant="secondary" onClick={() => setIsFormModalOpen(true)}>
          {t("lab.buttonFormModal")}
        </Button>
      </div>
      {infoResult ? (
        <p className={styles.result}>{t("lab.resultPrefix", { value: infoResult })}</p>
      ) : null}

      <SuccessDialog
        isOpen={isSuccessDialogOpen}
        onClose={() => setIsSuccessDialogOpen(false)}
        title={t("lab.successTitle")}
        text={t("lab.successText")}
      />
      <ErrorDialog
        isOpen={isErrorDialogOpen}
        onClose={() => setIsErrorDialogOpen(false)}
        title={t("lab.errorTitle")}
        text={t("lab.errorText")}
      />
      <AlertDialog
        isOpen={isAlertDialogOpen}
        onClose={() => setIsAlertDialogOpen(false)}
        title={t("lab.alertTitle")}
        text={t("lab.alertText")}
        confirmButtonText={t("lab.alertConfirm")}
        cancelButtonText={t("lab.alertCancel")}
        onConfirm={() => setInfoResult(t("lab.alertConfirmedResult"))}
      />
      <LoadingDialog
        isOpen={isLoadingDialogOpen}
        onClose={() => setIsLoadingDialogOpen(false)}
        title={t("lab.loadingTitle")}
        text={t("lab.loadingText")}
      />
      <FormModal
        isOpen={isFormModalOpen}
        onClose={() => setIsFormModalOpen(false)}
        title={t("lab.formModalTitle")}
      >
        <div className={styles.formBody}>
          <p>{t("lab.formModalBody")}</p>
          <Button variant="primary" onClick={() => setIsFormModalOpen(false)}>
            {t("lab.close")}
          </Button>
        </div>
      </FormModal>
    </div>
  );
}
