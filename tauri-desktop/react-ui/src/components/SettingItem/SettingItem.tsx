import { useRef, useState } from "react";
import type { IconType } from "react-icons";
import { FiExternalLink } from "react-icons/fi";
import { IoChevronForward } from "react-icons/io5";
import { MdUnfoldMore } from "react-icons/md";

import { Button } from "@/components/Button";
import { SelectionMenu } from "@/components/SelectionMenu";
import { Toggle } from "@/components/Toggle";

import * as styles from "./SettingItem.styles";

type SettingItemBase = {
  icon?: IconType;
  title: string;
  description?: string;
  danger?: boolean;
};

type NavigationSettingItem = SettingItemBase & {
  type: "navigation";
  onClick?: () => void;
  /** 現在値。あるときだけシェブロンの左に出す */
  value?: string;
};

/** 外部ブラウザ等で開くリンク */
type ExternalLinkSettingItem = SettingItemBase & {
  type: "externalLink";
  onClick: () => void;
};

type SelectionSettingItem = SettingItemBase & {
  type: "selection";
  value: string;
  options: readonly string[];
  onSelectionChanged: (value: string) => void;
  disabled?: boolean;
};

type ToggleSettingItem = SettingItemBase & {
  type: "toggle";
  /** `null` = unknown / loading (neutral toggle, not OFF). */
  toggleValue: boolean | null;
  onToggleChanged: (value: boolean) => void;
  disabled?: boolean;
};

type ActionSettingItem = SettingItemBase & {
  type: "action";
  onClick?: () => void;
};

/** 行全体は非インタラクティブ。末尾の Button で操作する */
type ButtonSettingItem = SettingItemBase & {
  type: "button";
  buttonLabel: string;
  onClick: () => void;
  disabled?: boolean;
};

export type SettingItemProps =
  | NavigationSettingItem
  | ExternalLinkSettingItem
  | SelectionSettingItem
  | ToggleSettingItem
  | ActionSettingItem
  | ButtonSettingItem;

function renderTrailing(props: SettingItemProps) {
  switch (props.type) {
    case "navigation":
      if (!props.value) {
        return (
          <span className={styles.chevron} aria-hidden>
            <IoChevronForward />
          </span>
        );
      }
      return (
        <span className={styles.trailing}>
          <span className={styles.value}>{props.value}</span>
          <span className={styles.chevron} aria-hidden>
            <IoChevronForward />
          </span>
        </span>
      );
    case "externalLink":
      return (
        <span className={styles.chevron} aria-hidden>
          <FiExternalLink />
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
          disabled={props.disabled}
          aria-label={props.title}
        />
      );
    case "button":
      return (
        <Button variant="neutral" disabled={props.disabled} onClick={props.onClick}>
          {props.buttonLabel}
        </Button>
      );
    case "action":
      return null;
  }
}

export function SettingItem(props: SettingItemProps) {
  const { icon: Icon, title, description, danger = false, type } = props;
  const triggerRef = useRef<HTMLButtonElement>(null);
  const [isMenuOpen, setIsMenuOpen] = useState(false);

  const tone = danger ? "danger" : "default";

  const content = (
    <>
      {Icon ? (
        <span className={styles.icon({ tone })} aria-hidden>
          <Icon />
        </span>
      ) : null}
      <span className={styles.textStack}>
        {description ? (
          <span className={styles.textGroup}>
            <span className={styles.title({ tone })}>{title}</span>
            <span className={styles.description}>{description}</span>
          </span>
        ) : (
          <span className={styles.title({ tone })}>{title}</span>
        )}
      </span>
      {renderTrailing(props)}
    </>
  );

  if (type === "toggle" || type === "button") {
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
          disabled={props.disabled}
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
