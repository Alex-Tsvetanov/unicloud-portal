import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ConfirmButton,
  DataTable,
  EmptyState,
  Flash,
  PageHeader,
  SeatsMeter,
  StatusBadge,
  type Column,
  type FlashMessage,
} from "@unicloud/ui";
import { useState } from "react";
import { Link, useParams } from "react-router";
import { api, errorText, type Schemas } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { NotFoundPage } from "./NotFound";

type Row = Schemas["RosterEntry"];

export function TeachOfferingPage() {
  const id = Number(useParams().id);
  const valid = Number.isInteger(id) && id > 0;
  const qc = useQueryClient();
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const query = useQuery({ queryKey: keys.roster(id), queryFn: () => api.roster(id), enabled: valid });

  const reject = useMutation({
    mutationFn: api.reject,
    onSuccess: () => {
      setFlash({ tone: "ok", text: "Записването е отхвърлено." });
      void qc.invalidateQueries({ queryKey: keys.roster(id) });
      void qc.invalidateQueries({ queryKey: keys.teach });
    },
    onError: (e) => setFlash({ tone: "error", text: errorText(e) }),
  });

  if (!valid) return <NotFoundPage />;

  const columns: Column<Row>[] = [
    { key: "name", header: "Студент", render: (r) => <Link to={`/students/${encodeURIComponent(r.student_sub)}`}>{r.name}</Link> },
    { key: "fn", header: "Фак. номер", render: (r) => <span className="uc-mono">{r.faculty_number}</span> },
    { key: "status", header: "Статус", render: (r) => <StatusBadge status={r.status.toLowerCase()} /> },
    {
      key: "actions",
      header: <span className="uc-muted">Действие</span>,
      render: (r) =>
        r.status === "ENROLLED" ? (
          <ConfirmButton
            question="Да отхвърля ли записването?"
            busy={reject.isPending && reject.variables === r.enrollment_id}
            onConfirm={() => reject.mutate(r.enrollment_id)}
          >
            Отхвърли
          </ConfirmButton>
        ) : null,
    },
  ];

  return (
    <QueryState query={query}>
      {({ offering, roster }) => (
        <>
          <PageHeader
            kicker={`${offering.code} · ${offering.semester}`}
            title={offering.title}
            actions={<SeatsMeter taken={offering.enrolled} capacity={offering.capacity} />}
          />
          <p>
            <Link to="/teach">← Всички курсове</Link>
          </p>
          <Flash message={flash} onClose={() => setFlash(null)} />
          <DataTable
            columns={columns}
            rows={roster}
            rowKey={(r) => r.enrollment_id}
            empty={<EmptyState title="Още няма записани студенти" />}
          />
        </>
      )}
    </QueryState>
  );
}
