import type { MouseEventHandler } from "react";

import { cx } from "@/styled-system/css";

import * as styles from "./Toggle.styles";

type ToggleProps = {
  /** `null` = unknown / loading (neutral, not OFF). */
  checked: boolean | null;
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
  const indeterminate = checked === null;
  const handleClick: MouseEventHandler<HTMLButtonElement> = (event) => {
    event.stopPropagation();
    if (disabled || indeterminate) {
      return;
    }
    onChange(!checked);
  };

  // Unknown: do not use role=switch (aria-checked="mixed" is invalid and UAs treat it as OFF).
  if (indeterminate) {
    return (
      <button
        type="button"
        aria-busy={true}
        aria-label={ariaLabel}
        disabled
        className={styles.track({ checked: "indeterminate" })}
        onClick={handleClick}
      >
        <span className={cx(styles.thumb, styles.thumbIndeterminate)} />
      </button>
    );
  }

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
