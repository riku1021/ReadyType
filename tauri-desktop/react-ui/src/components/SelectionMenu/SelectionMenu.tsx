import {
  type CSSProperties,
  type RefObject,
  useEffect,
  useLayoutEffect,
  useRef,
  useState,
} from "react";
import { createPortal } from "react-dom";

import { cx } from "@/styled-system/css";

import * as styles from "./SelectionMenu.styles";

type SelectionMenuProps = {
  isOpen: boolean;
  onClose: () => void;
  anchorRef: RefObject<HTMLElement | null>;
  label: string;
  options: readonly string[];
  currentSelection: string;
  onSelectionChanged: (value: string) => void;
};

type MenuPosition = {
  top: number;
  left: number;
};

const VIEWPORT_MARGIN = 12;
const GAP = 4;

function computePosition(anchor: HTMLElement, menu: HTMLElement): MenuPosition {
  const anchorRect = anchor.getBoundingClientRect();
  const menuRect = menu.getBoundingClientRect();
  const maxLeft = window.innerWidth - menuRect.width - VIEWPORT_MARGIN;
  const maxTop = window.innerHeight - menuRect.height - VIEWPORT_MARGIN;

  // トリガー右端にメニュー右端を揃える
  const left = Math.min(Math.max(VIEWPORT_MARGIN, anchorRect.right - menuRect.width), maxLeft);
  let top = anchorRect.bottom + GAP;

  if (top + menuRect.height > window.innerHeight - VIEWPORT_MARGIN) {
    top = anchorRect.top - menuRect.height - GAP;
  }

  top = Math.min(Math.max(VIEWPORT_MARGIN, top), maxTop);

  return { top, left };
}

export function SelectionMenu({
  isOpen,
  onClose,
  anchorRef,
  label,
  options,
  currentSelection,
  onSelectionChanged,
}: SelectionMenuProps) {
  const menuRef = useRef<HTMLDivElement>(null);
  const [position, setPosition] = useState<MenuPosition | null>(null);

  useLayoutEffect(() => {
    if (!isOpen) {
      setPosition(null);
      return;
    }

    const anchor = anchorRef.current;
    const menu = menuRef.current;
    if (!anchor || !menu) {
      return;
    }

    const update = () => {
      setPosition(computePosition(anchor, menu));
    };

    update();
    window.addEventListener("resize", update);
    window.addEventListener("scroll", update, true);
    return () => {
      window.removeEventListener("resize", update);
      window.removeEventListener("scroll", update, true);
    };
  }, [isOpen, anchorRef]);

  useEffect(() => {
    if (!isOpen) {
      return;
    }

    const handlePointerDown = (event: PointerEvent) => {
      const target = event.target;
      if (!(target instanceof Node)) {
        return;
      }
      if (menuRef.current?.contains(target)) {
        return;
      }
      if (anchorRef.current?.contains(target)) {
        return;
      }
      onClose();
    };

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        onClose();
      }
    };

    document.addEventListener("pointerdown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    return () => {
      document.removeEventListener("pointerdown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
    };
  }, [isOpen, onClose, anchorRef]);

  if (!isOpen) {
    return null;
  }

  const panelStyle: CSSProperties | undefined = position
    ? { top: position.top, left: position.left }
    : { top: 0, left: 0, visibility: "hidden" };

  return createPortal(
    <div
      ref={menuRef}
      className={styles.panel}
      style={panelStyle}
      role="listbox"
      aria-label={label}
    >
      <ul className={styles.list}>
        {options.map((option) => {
          const selected = option === currentSelection;
          return (
            <li key={option} role="presentation">
              <button
                type="button"
                role="option"
                aria-selected={selected}
                className={cx(styles.option, selected ? styles.optionSelected : undefined)}
                onClick={() => {
                  onSelectionChanged(option);
                  onClose();
                }}
              >
                {option}
              </button>
            </li>
          );
        })}
      </ul>
    </div>,
    document.body
  );
}
