import type { MouseEventHandler } from "react";

import { cx } from "@/styled-system/css";

import * as styles from "./Toggle.styles";

type ToggleProps = {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  "aria-label"?: string;
};

export function Toggle({
  checked,
  onChange,
  disabled = false,
  "aria-label": ariaLabel,
}: ToggleProps) {
  const handleClick: MouseEventHandler<HTMLButtonElement> = (event) => {
    event.stopPropagation();
    if (disabled) {
      return;
    }
    onChange(!checked);
  };

  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={ariaLabel}
      disabled={disabled}
      className={styles.track({ checked })}
      onClick={handleClick}
    >
      <span className={cx(styles.thumb, checked ? styles.thumbChecked : undefined)} />
    </button>
  );
}
