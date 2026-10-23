import { useRef } from "react";
import { useTranslation } from "react-i18next";
import { FaSearch } from "react-icons/fa";
import { IoIosClose } from "react-icons/io";

import { cx } from "@/styled-system/css";

import * as styles from "./SearchBar.styles";

type SearchBarProps = {
  value: string;
  onChange: (value: string) => void;
  placeholder: string;
  "aria-label": string;
  /** 省略時は `common.searchClearAria` */
  clearAriaLabel?: string;
  /** 親の横幅いっぱいに広げる */
  fullWidth?: boolean;
  className?: string;
};

export function SearchBar({
  value,
  onChange,
  placeholder,
  "aria-label": ariaLabel,
  clearAriaLabel,
  fullWidth = false,
  className,
}: SearchBarProps) {
  const { t } = useTranslation();
  const inputRef = useRef<HTMLInputElement>(null);
  const hasValue = value.length > 0;
  const resolvedClearAriaLabel = clearAriaLabel ?? t("common.searchClearAria");

  const clear = () => {
    onChange("");
    inputRef.current?.focus();
  };

  return (
    <div className={cx(styles.root({ fullWidth }), className)}>
      <input
        ref={inputRef}
        className={styles.input}
        type="search"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        placeholder={placeholder}
        aria-label={ariaLabel}
      />
      {hasValue ? (
        <button
          type="button"
          className={styles.clearButton}
          onClick={clear}
          aria-label={resolvedClearAriaLabel}
        >
          <IoIosClose className={styles.clearGlyph} aria-hidden="true" />
        </button>
      ) : (
        <span className={styles.searchIcon} aria-hidden="true">
          <FaSearch className={styles.searchGlyph} />
        </span>
      )}
    </div>
  );
}
