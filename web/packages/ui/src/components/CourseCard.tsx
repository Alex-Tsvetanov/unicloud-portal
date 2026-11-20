import type { ReactNode } from "react";
import { cx } from "../cx";
import { StatusBadge } from "./Badges";
import { SeatsMeter } from "./SeatsMeter";

export interface CourseCardProps {
  code?: string;
  title: ReactNode;
  description?: ReactNode;
  /** Short facts, for example faculty, teacher or credits. */
  meta?: ReactNode[];
  seats?: { taken: number; capacity: number };
  status?: string;
  /** Buttons at the bottom right. */
  actions?: ReactNode;
  className?: string;
}

export function CourseCard({ code, title, description, meta, seats, status, actions, className }: CourseCardProps) {
  const hasFoot = seats != null || actions != null;
  return (
    <article className={cx("uc-card", "uc-course", className)}>
      {(code != null || status != null) && (
        <div className="uc-row" style={{ justifyContent: "space-between" }}>
          {code != null && <span className="uc-course-code">{code}</span>}
          {status != null && <StatusBadge status={status} />}
        </div>
      )}
      <h3 className="uc-course-title">{title}</h3>
      {meta && meta.length > 0 && (
        <ul className="uc-course-meta">
          {meta.map((m, i) => (
            <li key={i}>{m}</li>
          ))}
        </ul>
      )}
      {description != null && <p className="uc-course-desc">{description}</p>}
      {hasFoot && (
        <div className="uc-card-foot">
          {seats ? <SeatsMeter taken={seats.taken} capacity={seats.capacity} /> : <span />}
          {actions != null && <div className="uc-row">{actions}</div>}
        </div>
      )}
    </article>
  );
}
