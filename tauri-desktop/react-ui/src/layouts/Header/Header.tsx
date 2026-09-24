import { useRouterState } from "@tanstack/react-router";
import { getNavLinkByPath } from "@/features/navigation";

import * as styles from "./Header.styles";

export function Header() {
  const { title, pathname } = useRouterState({
    select: (state) => {
      const matches = [...state.matches].reverse();
      return {
        title: matches.find((item) => item.staticData.title)?.staticData.title ?? "",
        pathname: state.location.pathname,
      };
    },
  });

  const navLink = getNavLinkByPath(pathname);
  const TitleIcon = navLink?.icon;

  return (
    <header className={styles.header}>
      <div className={styles.titleMain}>
        {TitleIcon ? (
          <span className={styles.titleIconBox}>
            <TitleIcon className={styles.titleIcon} aria-hidden="true" />
          </span>
        ) : null}
        <p className={styles.title}>{title}</p>
      </div>
    </header>
  );
}
