import { Link } from "@tanstack/react-router";
import { ThemeToggle } from "@/components/ThemeToggle";
import { navLinks } from "@/features/navigation";

import * as styles from "./Drawer.styles";

export function Drawer() {
  return (
    <aside className={styles.drawer}>
      <div className={styles.drawerCard}>
        <Link to="/" className={styles.logoLink}>
          <span className={styles.logoImageBox}>
            <img src="/pngs/logo.png" alt="" className={styles.logoImage} />
          </span>
          <span className={styles.logoText}>
            <span className={styles.logoAccent}>R</span>eady
            <span className={styles.logoAccent}>T</span>ype
          </span>
        </Link>
        <nav className={styles.navLinks} aria-label="メインナビゲーション">
          {navLinks.map((link) => {
            const IconComponent = link.icon;
            return (
              <Link key={link.to} to={link.to} className={styles.navLink}>
                <span className={styles.navLinkHighlight} data-part="highlight" aria-hidden />
                <div className={styles.navLinkContent}>
                  <span className={styles.navIconBox}>
                    <IconComponent className={styles.navIcon} />
                  </span>
                  <span className={styles.navText}>{link.label}</span>
                </div>
              </Link>
            );
          })}
        </nav>
        <div className={styles.themeToggleContainer}>
          <ThemeToggle />
        </div>
      </div>
    </aside>
  );
}
