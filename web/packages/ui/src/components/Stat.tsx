import type { ReactNode } from "react";
import { cx } from "../cx";

export interface StatProps {
  label: ReactNode;
  value: ReactNode;
  hint?: ReactNode;
  className?: string;
}

export function Stat({ label, value, hint, className }: StatProps) {
  return (
    <div className={cx("uc-stat", className)}>
      <span className="uc-stat-label">{label}</span>
      <span className="uc-stat-value">{value}</span>
      {hint != null && <span className="uc-stat-hint">{hint}</span>}
    </div>
  );
}
