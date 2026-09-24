import { useRef, useState } from "react";
import type { IconType } from "react-icons";
import { IoChevronForward } from "react-icons/io5";
import { MdUnfoldMore } from "react-icons/md";

import { SelectionMenu } from "@/components/SelectionMenu";
import { Toggle } from "@/components/Toggle";
import { cx } from "@/styled-system/css";

import * as styles from "./SettingItem.styles";

type SettingItemBase = {
  icon: IconType;
  title: string;
  danger?: boolean;
};

type NavigationSettingItem = SettingItemBase & {
  type: "navigation";
  onClick?: () => void;
};

type SelectionSettingItem = SettingItemBase & {
  type: "selection";
  value: string;
  options: readonly string[];
  onSelectionChanged: (value: string) => void;
};

type ToggleSettingItem = SettingItemBase & {
  type: "toggle";
  toggleValue: boolean;
  onToggleChanged: (value: boolean) => void;
};

type ActionSettingItem = SettingItemBase & {
  type: "action";
  onClick?: () => void;
};

export type SettingItemProps =
  | NavigationSettingItem
  | SelectionSettingItem
  | ToggleSettingItem
  | ActionSettingItem;

function renderTrailing(props: SettingItemProps) {
  switch (props.type) {
    case "navigation":
      return (
        <span className={styles.chevron} aria-hidden>
          <IoChevronForward />
        </span>
      );
    case "selection":
      return (
        <span className={styles.trailing}>
          <span className={styles.value}>{props.value}</span>
          <span className={styles.chevron} aria-hidden>
            <MdUnfoldMore />
          </span>
        </span>
      );
    case "toggle":
      return (
        <Toggle
          checked={props.toggleValue}
          onChange={props.onToggleChanged}
          aria-label={props.title}
        />
      );
    case "action":
      return null;
  }
}

export function SettingItem(props: SettingItemProps) {
  const { icon: Icon, title, danger = false, type } = props;
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const content = (
    <>
      <span className={cx(styles.icon, danger ? styles.iconError : undefined)} aria-hidden>
        <Icon />
      </span>
      <span className={cx(styles.title, danger ? styles.titleError : undefined)}>{title}</span>
      {renderTrailing(props)}
    </>
  );

  if (type === "toggle") {
    return <div className={styles.row({ density: "compact", interactive: false })}>{content}</div>;
  }

  if (type === "selection") {
    return (
      <>
        <button
          ref={triggerRef}
          type="button"
          className={styles.row({ density: "default", interactive: true })}
          aria-haspopup="listbox"
          aria-expanded={isMenuOpen}
          onClick={() => setIsMenuOpen((open) => !open)}
        >
          {content}
        </button>
        <SelectionMenu
          isOpen={isMenuOpen}
          onClose={() => setIsMenuOpen(false)}
          anchorRef={triggerRef}
          label={title}
          options={props.options}
          currentSelection={props.value}
          onSelectionChanged={props.onSelectionChanged}
        />
      </>
    );
  }

  if (!props.onClick) {
    return <div className={styles.row({ density: "default", interactive: false })}>{content}</div>;
  }

  return (
    <button
      type="button"
      className={styles.row({ density: "default", interactive: true })}
      onClick={props.onClick}
    >
      {content}
    </button>
  );
}
