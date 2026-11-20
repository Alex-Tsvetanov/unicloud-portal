import { Card, PageHeader, RoleBadge } from "@unicloud/ui";
import { Link } from "react-router";
import { useMe } from "../api/hooks";

export function HomePage() {
  const { data: me } = useMe();
  if (!me) return null;
  const links = me.nav.filter((n) => n.path !== "/home");
  return (
    <>
      <PageHeader
        kicker={`Семестър ${me.semester}`}
        title={`Здравейте, ${me.user.given_name || me.user.name}`}
        lede="Това е вашият портал в UniCloud. Изберете раздел."
        actions={<RoleBadge role={me.user.role} label={me.user.role_label} />}
      />
      <div className="uc-grid">
        {links.map((n) => (
          <Card key={n.path} as="article" title={<Link to={n.path}>{n.label}</Link>} />
        ))}
      </div>
      {me.user.faculty && (
        <p className="uc-muted uc-small">
          Факултет: {me.user.faculty.name} ({me.user.faculty.code})
        </p>
      )}
    </>
  );
}
