import type { MouseEventHandler, ReactNode } from "react";

import { type ButtonLayout, type ButtonVariant, button, icon } from "./Button.styles";

export type { ButtonLayout, ButtonVariant };

type SharedButtonProps = {
  variant?: ButtonVariant;
  layout?: ButtonLayout;
  /** 親幅いっぱいに伸ばす */
  fullWidth?: boolean;
  disabled?: boolean;
  "aria-label"?: string;
  /** ラベル左のアイコン（なし可）。layout=icon のときは使わず children をアイコンとして扱う */
  iconLeft?: ReactNode;
  /** ラベル右のアイコン（なし可） */
  iconRight?: ReactNode;
  children: ReactNode;
};

/** フォーム送信。クリック処理は form の onSubmit に任せる */
type SubmitButtonProps = SharedButtonProps & {
  type: "submit";
  onClick?: never;
};

/** 通常操作。onClick 必須 */
type ActionButtonProps = SharedButtonProps & {
  type?: "button";
  onClick: MouseEventHandler<HTMLButtonElement>;
};

/** type="submit" か、onClick 必須の通常ボタンのどちらか一方 */
type ButtonProps = SubmitButtonProps | ActionButtonProps;

export function Button(props: ButtonProps) {
  const {
    variant = "primary",
    layout = "labeled",
    fullWidth = false,
    disabled,
    "aria-label": ariaLabel,
    iconLeft,
    iconRight,
    children,
  } = props;

  const isSubmit = props.type === "submit";
  const type = isSubmit ? "submit" : "button";
  const onClick = isSubmit ? undefined : props.onClick;

  return (
    <button
      type={type}
      disabled={disabled}
      onClick={onClick}
      aria-label={ariaLabel}
      className={button({ variant, layout, fullWidth })}
    >
      {layout === "icon" ? (
        <span className={icon} aria-hidden="true">
          {children}
        </span>
      ) : (
        <>
          {iconLeft ? (
            <span className={icon} aria-hidden="true">
              {iconLeft}
            </span>
          ) : null}
          {children}
          {iconRight ? (
            <span className={icon} aria-hidden="true">
              {iconRight}
            </span>
          ) : null}
        </>
      )}
    </button>
  );
}
