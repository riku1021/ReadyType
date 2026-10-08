import { useEffect, useState } from "react";
import { useTranslation } from "react-i18next";

import { Button } from "@/components/Button";
import { AlertDialog, ErrorDialog, LoadingDialog, SuccessDialog } from "@/components/Dialog";
import { FormModal } from "@/components/FormModal";
import { ensureLabSwitchRules, openLabSwitchWindow } from "@/features/inputSource/api";

import * as styles from "./LabPage.styles";
import { LAB_SWITCH_FIELDS } from "./labSwitchFields";

export function LabPage() {
  const { t } = useTranslation();
  const [infoResult, setInfoResult] = useState("");
  const [isSuccessDialogOpen, setIsSuccessDialogOpen] = useState(false);
  const [isErrorDialogOpen, setIsErrorDialogOpen] = useState(false);
  const [isAlertDialogOpen, setIsAlertDialogOpen] = useState(false);
  const [isLoadingDialogOpen, setIsLoadingDialogOpen] = useState(false);
  const [isFormModalOpen, setIsFormModalOpen] = useState(false);
  const [labRulesReady, setLabRulesReady] = useState(false);
  const [labRulesError, setLabRulesError] = useState<string | null>(null);
  const [labWindowError, setLabWindowError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void ensureLabSwitchRules()
      .then(() => {
        if (!cancelled) {
          setLabRulesReady(true);
          setLabRulesError(null);
        }
      })
      .catch((error: unknown) => {
        console.error("[lab] ensure_lab_switch_rules failed", error);
        if (!cancelled) {
          setLabRulesReady(false);
          setLabRulesError(t("lab.switchRulesEnsureFailed"));
        }
      });
    return () => {
      cancelled = true;
    };
  }, [t]);

  const handleOpenLabWindow = () => {
    setLabWindowError(null);
    void openLabSwitchWindow().catch((error: unknown) => {
      console.error("[lab] open_lab_switch_window failed", error);
      setLabWindowError(t("lab.openNativeWindowFailed"));
    });
  };

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

      <h2 className={styles.sectionTitle}>{t("lab.sectionSwitchTest")}</h2>
      <p className={styles.lead}>{t("lab.switchTestLead")}</p>
      {labRulesError ? <p className={styles.result}>{labRulesError}</p> : null}
      {!labRulesReady && !labRulesError ? (
        <p className={styles.result}>{t("lab.switchRulesEnsuring")}</p>
      ) : null}
      {labWindowError ? <p className={styles.result}>{labWindowError}</p> : null}
      <div className={styles.buttonGroup}>
        <Button variant="primary" onClick={handleOpenLabWindow} disabled={!labRulesReady}>
          {t("lab.openNativeWindow")}
        </Button>
      </div>
      <ul className={styles.fieldList}>
        {LAB_SWITCH_FIELDS.map((field, index) => {
          const ordinal = index + 1;
          const languageLabel = t(`lab.switchLanguage.${field.language}`);
          return (
            <li key={field.id} className={styles.fieldRow}>
              <span className={styles.fieldLabel}>
                {t("lab.switchFieldLabel", {
                  index: ordinal,
                  language: languageLabel,
                })}
              </span>
              <span className={styles.fieldHint}>{t("lab.switchFieldHint", { id: field.id })}</span>
            </li>
          );
        })}
      </ul>

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
