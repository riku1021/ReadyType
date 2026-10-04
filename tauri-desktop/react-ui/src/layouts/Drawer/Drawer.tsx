import { Link } from "@tanstack/react-router";
import { useTranslation } from "react-i18next";

import { BrandLogo } from "@/components/BrandLogo";
import { ThemeToggle } from "@/components/ThemeToggle";
import { SwitcherControls } from "@/features/inputSource";
import { navLinks } from "@/features/navigation";

import * as styles from "./Drawer.styles";

export function Drawer() {
  const { t } = useTranslation();

  return (
    <aside className={styles.drawer}>
      <div className={styles.drawerCard}>
        <Link to="/" className={styles.logoLink}>
          <BrandLogo />
        </Link>
        <nav className={styles.navLinks} aria-label={t("nav.mainNavigationAria")}>
          {navLinks.map((link) => {
            const IconComponent = link.icon;
            return (
              <Link key={link.to} to={link.to} className={styles.navLink}>
                <span className={styles.navLinkHighlight} data-part="highlight" aria-hidden />
                <div className={styles.navLinkContent}>
                  <span className={styles.navIconBox}>
                    <IconComponent className={styles.navIcon} />
                  </span>
                  <span className={styles.navText}>{t(link.labelKey)}</span>
                </div>
              </Link>
            );
          })}
        </nav>
        <div className={styles.controlsRow}>
          <ThemeToggle />
          <SwitcherControls />
        </div>
      </div>
    </aside>
  );
}
