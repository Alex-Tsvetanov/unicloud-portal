import type { ReactNode } from "react";
import { cx } from "../cx";

export interface StudentCardProps {
  name: string;
  /** Faculty number, email or other short facts. */
  meta?: ReactNode[];
  badges?: ReactNode;
  actions?: ReactNode;
  className?: string;
}

/** First letters of the first and last word, for the avatar. */
export function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  if (parts.length === 0) return "?";
  const first = parts[0][0];
  const last = parts.length > 1 ? parts[parts.length - 1][0] : "";
  return (first + last).toUpperCase();
}

export function StudentCard({ name, meta, badges, actions, className }: StudentCardProps) {
  return (
    <article className={cx("uc-card", "uc-student", className)}>
      <div className="uc-row">
        <span className="uc-avatar" aria-hidden="true">
          {initials(name)}
        </span>
        <div>
          <h3 className="uc-student-name">{name}</h3>
          {meta && meta.length > 0 && (
            <ul className="uc-student-meta">
              {meta.map((m, i) => (
                <li key={i}>{m}</li>
              ))}
            </ul>
          )}
        </div>
      </div>
      {badges != null && <div className="uc-row">{badges}</div>}
      {actions != null && (
        <div className="uc-card-foot">
          <span />
          {actions}
        </div>
      )}
    </article>
  );
}
