import { useTranslation } from "react-i18next";

import * as styles from "./UpdatePage.styles";

export function UpdatePage() {
  const { t } = useTranslation();

  return (
    <div className={styles.page}>
      <h1 className={styles.heading}>{t("update.heading")}</h1>
      <p className={styles.lead}>{t("update.lead")}</p>
    </div>
  );
}
