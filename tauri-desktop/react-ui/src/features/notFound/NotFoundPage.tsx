import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";
import { IoHome } from "react-icons/io5";

import NotFoundAnimation from "@/animations/NotFoundAnimation";
import { Button, button } from "@/components/Button";
import { cx } from "@/styled-system/css";

import * as styles from "./NotFoundPage.styles";

export function NotFoundPage() {
  const { t } = useTranslation();

  return (
    <div className={styles.notFoundContainer}>
      <NotFoundAnimation className={styles.animation} />
      <h2 className={styles.heading}>{t("notFound.heading")}</h2>
      <p className={styles.paragraph}>{t("notFound.body")}</p>
      <Link to="/apps" className={cx(button({ variant: "secondary" }), styles.homeLink)}>
        <IoHome className={styles.homeIcon} /> {t("notFound.goHome")}
      </Link>
      <div className={styles.backButtonWrap}>
        <Button variant="text" onClick={() => window.history.back()}>
          {t("notFound.goBack")}
        </Button>
      </div>
    </div>
  );
}
