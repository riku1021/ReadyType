import { cx } from "@/styled-system/css";

import * as styles from "./BrandLogo.styles";

type BrandLogoProps = {
  className?: string;
};

/** Drawer / Quick Set 共通の ReadyType ロゴ行 */
export function BrandLogo({ className }: BrandLogoProps) {
  return (
    <div className={cx(styles.logoRow, className)}>
      <span className={styles.logoImageBox}>
        <img src="/pngs/logo.png" alt="" className={styles.logoImage} />
      </span>
      <span className={styles.logoText}>
        <span className={styles.logoAccent}>R</span>eady
        <span className={styles.logoAccent}>T</span>ype
      </span>
    </div>
  );
}
