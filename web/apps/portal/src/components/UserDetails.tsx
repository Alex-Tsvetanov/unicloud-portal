import { Card, RoleBadge } from "@unicloud/ui";
import type { Schemas } from "../api/client";
import { ROLE_LABEL, STUDENT_STATUS } from "../labels";

interface Row {
  label: string;
  value: string | number;
}

function rowsFor(u: Schemas["DirectoryUser"]): Row[] {
  const rows: Row[] = [
    { label: "Имейл", value: u.email },
    { label: "Име", value: u.given_name },
    { label: "Фамилия", value: u.family_name },
  ];
  if (u.faculty) rows.push({ label: "Факултет", value: `${u.faculty.name} (${u.faculty.code})` });
  if (u.student) {
    rows.push(
      { label: "Факултетен номер", value: u.student.faculty_number },
      { label: "Специалност", value: u.student.specialty },
      { label: "Курс", value: u.student.study_year },
      { label: "Статус", value: STUDENT_STATUS[u.student.status] ?? u.student.status },
    );
  }
  return rows;
}

export function UserDetails({ user }: { user: Schemas["DirectoryUser"] }) {
  return (
    <Card title={user.name} aside={<RoleBadge role={user.role} label={ROLE_LABEL[user.role]} />}>
      <dl style={{ display: "grid", gridTemplateColumns: "max-content 1fr", gap: ".4rem 1.2rem", margin: 0 }}>
        {rowsFor(user).map((r) => (
          <div key={r.label} style={{ display: "contents" }}>
            <dt className="uc-muted">{r.label}</dt>
            <dd style={{ margin: 0 }}>{r.value}</dd>
          </div>
        ))}
      </dl>
    </Card>
  );
}
