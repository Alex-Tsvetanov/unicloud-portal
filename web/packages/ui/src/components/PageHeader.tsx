import type { ReactNode } from "react";
import { cx } from "../cx";

export interface PageHeaderProps {
  title: ReactNode;
  kicker?: ReactNode;
  lede?: ReactNode;
  /** Buttons or links on the right. */
  actions?: ReactNode;
  className?: string;
}

export function PageHeader({ title, kicker, lede, actions, className }: PageHeaderProps) {
  return (
    <div className={cx("uc-page-header", className)}>
      <div>
        {kicker != null && <p className="uc-kicker">{kicker}</p>}
        <h1 className="uc-h1">{title}</h1>
        {lede != null && <p className="uc-lede">{lede}</p>}
      </div>
      {actions != null && <div className="uc-row">{actions}</div>}
    </div>
  );
}
