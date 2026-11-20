import type { ButtonHTMLAttributes, Ref } from "react";
import { cx } from "../cx";

export type ButtonVariant = "primary" | "secondary" | "link" | "danger";
export type ButtonSize = "sm" | "md" | "lg";

export interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant;
  size?: ButtonSize;
  /** Disables the button and marks it busy while a request runs. */
  busy?: boolean;
  ref?: Ref<HTMLButtonElement>;
}

/** The class list of a button, for links that should look like one. */
export function buttonClass(variant: ButtonVariant = "primary", size: ButtonSize = "md"): string {
  return cx("uc-btn", `uc-btn-${variant}`, size !== "md" && `uc-btn-${size}`);
}

export function Button({
  variant = "primary",
  size = "md",
  busy = false,
  type = "button",
  className,
  disabled,
  children,
  ...rest
}: ButtonProps) {
  return (
    <button
      type={type}
      className={cx(buttonClass(variant, size), className)}
      disabled={disabled || busy}
      aria-busy={busy || undefined}
      {...rest}
    >
      {children}
    </button>
  );
}
