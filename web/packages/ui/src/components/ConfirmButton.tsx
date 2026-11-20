import { useEffect, useRef, useState, type ReactNode } from "react";
import { Button, type ButtonSize, type ButtonVariant } from "./Button";

export interface ConfirmButtonProps {
  /** Label of the first button. */
  children: ReactNode;
  /** Question shown in the second step. */
  question?: ReactNode;
  confirmLabel?: ReactNode;
  cancelLabel?: ReactNode;
  onConfirm: () => void;
  variant?: ButtonVariant;
  size?: ButtonSize;
  busy?: boolean;
  disabled?: boolean;
}

/** A two-step confirm inside the page. It never calls window.confirm. Escape cancels. */
export function ConfirmButton({
  children,
  question = "Сигурни ли сте?",
  confirmLabel = "Да",
  cancelLabel = "Не",
  onConfirm,
  variant = "danger",
  size = "sm",
  busy,
  disabled,
}: ConfirmButtonProps) {
  const [asking, setAsking] = useState(false);
  const confirmRef = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    if (asking) confirmRef.current?.focus();
  }, [asking]);

  if (!asking) {
    return (
      <Button variant={variant} size={size} busy={busy} disabled={disabled} onClick={() => setAsking(true)}>
        {children}
      </Button>
    );
  }

  const handleConfirm = () => {
    setAsking(false);
    onConfirm();
  };

  return (
    <span
      className="uc-confirm"
      role="group"
      onKeyDown={(e) => {
        if (e.key === "Escape") setAsking(false);
      }}
    >
      <span className="uc-confirm-q">{question}</span>
      <Button ref={confirmRef} variant={variant} size={size} onClick={handleConfirm}>
        {confirmLabel}
      </Button>
      <Button variant="secondary" size={size} onClick={() => setAsking(false)}>
        {cancelLabel}
      </Button>
    </span>
  );
}
