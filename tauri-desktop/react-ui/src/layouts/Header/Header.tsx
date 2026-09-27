import { Link, useRouterState } from "@tanstack/react-router";
import { MdChevronLeft } from "react-icons/md";

import { button } from "@/components/Button";
import { getNavLinkByPath } from "@/features/navigation";

import * as styles from "./Header.styles";

export function Header() {
  const { title, headerBack, pathname } = useRouterState({
    select: (state) => {
      const matches = [...state.matches].reverse();
      return {
        title: matches.find((item) => item.staticData.title)?.staticData.title ?? "",
        headerBack: matches.find((item) => item.staticData.headerBack)?.staticData.headerBack,
        pathname: state.location.pathname,
      };
    },
  });

  const navLink = getNavLinkByPath(pathname);
  const TitleIcon = navLink?.icon;

  return (
    <header className={styles.header}>
      <div className={styles.titleRow}>
        {headerBack ? (
          <Link to={headerBack.to} className={button({ variant: "secondary" })}>
            <MdChevronLeft className={styles.backButtonIcon} aria-hidden="true" />
            {headerBack.label}
          </Link>
        ) : null}
        <div className={styles.titleMain}>
          {TitleIcon ? (
            <span className={styles.titleIconBox}>
              <TitleIcon className={styles.titleIcon} aria-hidden="true" />
            </span>
          ) : null}
          <p className={styles.title}>{title}</p>
        </div>
      </div>
    </header>
  );
}
