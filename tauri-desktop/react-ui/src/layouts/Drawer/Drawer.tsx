import { Link } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { useTranslation } from "react-i18next";

import { BrandLogo } from "@/components/BrandLogo";
import { ThemeToggle } from "@/components/ThemeToggle";
import {
  anyAppHasMissingInputSource,
  inputSourceSourcesAtom,
  inputSourceStateAtom,
  SwitcherControls,
} from "@/features/inputSource";
import { navLinks } from "@/features/navigation";

import * as styles from "./Drawer.styles";

export function Drawer() {
  const { t } = useTranslation();
  const feature = useAtomValue(inputSourceStateAtom);
  const sources = useAtomValue(inputSourceSourcesAtom);
  const appsHaveMissingInputSource =
    sources.status === "ready" &&
    anyAppHasMissingInputSource(
      feature.rules,
      feature.settings?.languagePreferences ?? {},
      sources.sources
    );

  return (
    <aside className={styles.drawer}>
      <div className={styles.drawerCard}>
        <Link to="/" className={styles.logoLink}>
          <BrandLogo />
        </Link>
        <nav className={styles.navLinks} aria-label={t("nav.mainNavigationAria")}>
          {navLinks.map((link) => {
            const IconComponent = link.icon;
            const showMissingInputSourceBadge = link.to === "/apps" && appsHaveMissingInputSource;
            return (
              <Link key={link.to} to={link.to} className={styles.navLink}>
                <span className={styles.navLinkHighlight} data-part="highlight" aria-hidden />
                <div className={styles.navLinkContent}>
                  <span className={styles.navIconBox}>
                    <span className={styles.navIconWrap}>
                      <IconComponent className={styles.navIcon} />
                      {showMissingInputSourceBadge ? (
                        <span
                          className={styles.navIconWarningBadge}
                          role="img"
                          aria-label={t("nav.appsMissingInputSourceAria")}
                        >
                          <span aria-hidden>!</span>
                        </span>
                      ) : null}
                    </span>
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
