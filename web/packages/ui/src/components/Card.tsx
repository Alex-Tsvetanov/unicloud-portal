import type { HTMLAttributes, ReactNode } from "react";
import { cx } from "../cx";

export interface CardProps extends Omit<HTMLAttributes<HTMLElement>, "title"> {
  title?: ReactNode;
  /** Small content on the right of the title, for example a badge or a link. */
  aside?: ReactNode;
  as?: "section" | "article" | "div";
}

export function Card({ title, aside, as: Tag = "section", className, children, ...rest }: CardProps) {
  return (
    <Tag className={cx("uc-card", className)} {...rest}>
      {(title != null || aside != null) && (
        <div className="uc-card-head">
          {title != null && <h2 className="uc-h2">{title}</h2>}
          {aside}
        </div>
      )}
      {children}
    </Tag>
  );
}
