import { Link, useRouterState } from "@tanstack/react-router";
import type { ParseKeys } from "i18next";
import { useTranslation } from "react-i18next";
import { MdChevronLeft } from "react-icons/md";

import { button } from "@/components/Button";
import { getNavLinkByPath } from "@/features/navigation";

import * as styles from "./Header.styles";

export function Header() {
  const { t } = useTranslation();
  const { title, headerBack, pathname } = useRouterState({
    select: (state) => {
      const matches = [...state.matches].reverse();
      return {
        title: matches.find((item) => item.staticData.title)?.staticData.title,
        headerBack: matches.find((item) => item.staticData.headerBack)?.staticData.headerBack,
        pathname: state.location.pathname,
      };
    },
  });

  const navLink = getNavLinkByPath(pathname);
  const TitleIcon = navLink?.icon;
  const titleKey = title as ParseKeys<"translation"> | undefined;
  const backLabelKey = headerBack?.label as ParseKeys<"translation"> | undefined;

  return (
    <header className={styles.header}>
      <div className={styles.titleRow}>
        {headerBack && backLabelKey ? (
          <Link to={headerBack.to} className={button({ variant: "secondary" })}>
            <MdChevronLeft className={styles.backButtonIcon} aria-hidden="true" />
            {t(backLabelKey)}
          </Link>
        ) : null}
        <div className={styles.titleMain}>
          {TitleIcon ? (
            <span className={styles.titleIconBox}>
              <TitleIcon className={styles.titleIcon} aria-hidden="true" />
            </span>
          ) : null}
          <p className={styles.title}>{titleKey ? t(titleKey) : ""}</p>
        </div>
      </div>
    </header>
  );
}
