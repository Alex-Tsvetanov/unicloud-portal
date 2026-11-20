import { useQuery } from "@tanstack/react-query";
import { Card, DataTable, EmptyState, PageHeader, SeatsMeter, Stat, StatusBadge, type Column } from "@unicloud/ui";
import { Link } from "react-router";
import { api, type Schemas } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { STUDENT_STATUS } from "../labels";

const offeringCols: Column<Schemas["FacultyOffering"]>[] = [
  { key: "code", header: "Код", render: (r) => <span className="uc-mono">{r.code}</span> },
  { key: "title", header: "Курс", render: (r) => r.title },
  { key: "teacher", header: "Преподавател", render: (r) => r.teacher },
  { key: "seats", header: "Места", render: (r) => <SeatsMeter taken={r.enrolled} capacity={r.capacity} /> },
];

const studentCols: Column<Schemas["FacultyStudent"]>[] = [
  { key: "name", header: "Име", render: (r) => <Link to={`/students/${encodeURIComponent(r.sub)}`}>{r.name}</Link> },
  { key: "fn", header: "Фак. номер", render: (r) => <span className="uc-mono">{r.faculty_number}</span> },
  { key: "spec", header: "Специалност", render: (r) => r.specialty },
  { key: "year", header: "Курс", numeric: true, render: (r) => r.study_year },
  {
    key: "status",
    header: "Статус",
    render: (r) => <StatusBadge status={r.status === "ACTIVE" ? "active" : r.status} label={STUDENT_STATUS[r.status]} />,
  },
];

export function FacultyPage() {
  const query = useQuery({ queryKey: keys.faculty, queryFn: api.faculty });
  return (
    <QueryState query={query}>
      {(f) => (
        <>
          <PageHeader kicker={`Семестър ${f.semester}`} title={f.faculty.name} lede={`Код на факултета: ${f.faculty.code}`} />
          <div className="uc-grid" style={{ marginBottom: "1.4rem" }}>
            <Stat label="Курсове" value={f.offerings.length} />
            <Stat label="Студенти" value={f.students.length} />
          </div>
          <Card title="Курсове този семестър">
            <DataTable columns={offeringCols} rows={f.offerings} rowKey={(r) => r.code} empty={<EmptyState title="Няма курсове" />} />
          </Card>
          <Card title="Студенти">
            <DataTable columns={studentCols} rows={f.students} rowKey={(r) => r.sub} empty={<EmptyState title="Няма студенти" />} />
          </Card>
        </>
      )}
    </QueryState>
  );
}
