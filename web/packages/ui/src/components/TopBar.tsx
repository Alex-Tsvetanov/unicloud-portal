import type { ReactNode } from "react";
import { cx } from "../cx";

export interface NavItem {
  label: string;
  href: string;
  active?: boolean;
}

export interface RenderLinkArgs {
  item: NavItem;
  className?: string;
  children: ReactNode;
}

export interface TopBarProps {
  /** Product name shown next to the mark. */
  brand?: string;
  /** One or two letters inside the brand mark. */
  brandMark?: string;
  brandHref?: string;
  items: NavItem[];
  /** Lets a router render its own link component. Defaults to a plain <a>. */
  renderLink?: (args: RenderLinkArgs) => ReactNode;
  /** Right-hand slot, for example the user's name and a sign-out form. */
  right?: ReactNode;
  /** Accessible name of the nav landmark. */
  navLabel?: string;
  className?: string;
}

function defaultLink({ item, className, children }: RenderLinkArgs): ReactNode {
  return (
    <a href={item.href} className={className} aria-current={item.active ? "page" : undefined}>
      {children}
    </a>
  );
}

export function TopBar({
  brand = "UniCloud",
  brandMark = "U",
  brandHref = "/",
  items,
  renderLink = defaultLink,
  right,
  navLabel = "Основна навигация",
  className,
}: TopBarProps) {
  return (
    <header className={cx("uc-topbar", className)}>
      {renderLink({
        item: { label: brand, href: brandHref },
        className: "uc-brand",
        children: (
          <>
            <span className="uc-brand-mark" aria-hidden="true">{brandMark}</span>
            <span>{brand}</span>
          </>
        ),
      })}
      <nav className="uc-nav" aria-label={navLabel}>
        {items.map((item) => (
          <span key={item.href}>{renderLink({ item, children: item.label })}</span>
        ))}
      </nav>
      {right != null && <div className="uc-who">{right}</div>}
    </header>
  );
}
