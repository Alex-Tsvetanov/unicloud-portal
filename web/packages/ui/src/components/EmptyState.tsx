import type { ReactNode } from "react";
import { cx } from "../cx";

export interface EmptyStateProps {
  title: ReactNode;
  children?: ReactNode;
  action?: ReactNode;
  className?: string;
}

export function EmptyState({ title, children, action, className }: EmptyStateProps) {
  return (
    <div className={cx("uc-empty", className)}>
      <p className="uc-empty-title">{title}</p>
      {children != null && <p>{children}</p>}
      {action}
    </div>
  );
}
