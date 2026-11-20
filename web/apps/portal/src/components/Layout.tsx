import { Alert, Button, RoleBadge, TopBar } from "@unicloud/ui";
import { Link, Outlet, useLocation } from "react-router";
import { errorText } from "../api/client";
import { useMe } from "../api/hooks";

export function Layout() {
  const me = useMe();
  const { pathname } = useLocation();

  const items = (me.data?.nav ?? []).map((n) => ({
    label: n.label,
    href: n.path,
    active: pathname === n.path || pathname.startsWith(`${n.path}/`),
  }));

  const right = me.data ? (
    <>
      <span>{me.data.user.name}</span>
      <RoleBadge role={me.data.user.role} label={me.data.user.role_label} />
      <form method="post" action="/logout">
        <input type="hidden" name="csrf" value={me.data.csrf} />
        <Button type="submit" variant="secondary" size="sm">
          Изход
        </Button>
      </form>
    </>
  ) : null;

  return (
    <>
      <TopBar
        brand="UniCloud"
        brandHref="/home"
        items={items}
        right={right}
        renderLink={({ item, className, children }) => (
          <Link to={item.href} className={className} aria-current={item.active ? "page" : undefined}>
            {children}
          </Link>
        )}
      />
      <main id="main" className="uc-main">
        {me.isPending && <p className="uc-muted">Зареждане…</p>}
        {me.isError && <Alert tone="error">{errorText(me.error)}</Alert>}
        {me.isSuccess && <Outlet />}
      </main>
    </>
  );
}
