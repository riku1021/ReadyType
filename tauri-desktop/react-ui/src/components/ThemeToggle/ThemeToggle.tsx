import { useAtom } from "jotai";
import { MdDarkMode, MdLightMode } from "react-icons/md";
import { toggleThemeAtom } from "@/atoms/theme";

import * as styles from "./ThemeToggle.styles";

export function ThemeToggle() {
  const [, toggleTheme] = useAtom(toggleThemeAtom);

  return (
    <button
      type="button"
      className={styles.button}
      onClick={toggleTheme}
      aria-label="テーマ切り替え"
    >
      <MdLightMode className={styles.iconSun} />
      <MdDarkMode className={styles.iconMoon} />
    </button>
  );
}
