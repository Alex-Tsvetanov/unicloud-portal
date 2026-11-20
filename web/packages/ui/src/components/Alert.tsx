import type { ReactNode } from "react";
import { cx } from "../cx";

export type AlertTone = "ok" | "error" | "info";

export interface AlertProps {
  tone?: AlertTone;
  children: ReactNode;
  /** Shows a close button when given. */
  onClose?: () => void;
  closeLabel?: string;
  className?: string;
}

export function Alert({ tone = "info", children, onClose, closeLabel = "Затвори", className }: AlertProps) {
  return (
    <div className={cx("uc-alert", `uc-alert-${tone}`, className)} role={tone === "error" ? "alert" : "status"}>
      <div className="uc-alert-body">{children}</div>
      {onClose && (
        <button type="button" className="uc-alert-close" onClick={onClose} aria-label={closeLabel}>
          ×
        </button>
      )}
    </div>
  );
}

export interface FlashMessage {
  tone: AlertTone;
  text: ReactNode;
}

export interface FlashProps {
  message: FlashMessage | null | undefined;
  onClose?: () => void;
  className?: string;
}

/** Renders one result message, or nothing. */
export function Flash({ message, onClose, className }: FlashProps) {
  if (!message) return null;
  return (
    <Alert tone={message.tone} onClose={onClose} className={className}>
      {message.text}
    </Alert>
  );
}
