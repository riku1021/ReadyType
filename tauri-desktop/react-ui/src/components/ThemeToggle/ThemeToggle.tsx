import { useAtom } from "jotai";
import { useTranslation } from "react-i18next";
import { MdDarkMode, MdLightMode } from "react-icons/md";

import { toggleThemeAtom } from "@/atoms/theme";

import * as styles from "./ThemeToggle.styles";

export function ThemeToggle() {
  const { t } = useTranslation();
  const [, toggleTheme] = useAtom(toggleThemeAtom);

  return (
    <button
      type="button"
      className={styles.button}
      onClick={toggleTheme}
      aria-label={t("theme.toggleAria")}
    >
      <MdLightMode className={styles.iconSun} />
      <MdDarkMode className={styles.iconMoon} />
    </button>
  );
}
