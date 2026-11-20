import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  ConfirmButton,
  DataTable,
  EmptyState,
  Flash,
  PageHeader,
  StatusBadge,
  buttonClass,
  type Column,
  type FlashMessage,
} from "@unicloud/ui";
import { useState } from "react";
import { Link } from "react-router";
import { api, errorText, type Schemas } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";

type Row = Schemas["MyEnrollment"];

export function EnrollmentsPage() {
  const qc = useQueryClient();
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const query = useQuery({ queryKey: keys.enrollments, queryFn: api.enrollments });

  const cancel = useMutation({
    mutationFn: api.cancelEnrollment,
    onSuccess: () => {
      setFlash({ tone: "ok", text: "Записването е отказано." });
      void qc.invalidateQueries({ queryKey: keys.enrollments });
      void qc.invalidateQueries({ queryKey: ["courses"] });
    },
    onError: (e) => setFlash({ tone: "error", text: errorText(e) }),
  });

  const columns: Column<Row>[] = [
    { key: "code", header: "Код", render: (r) => <span className="uc-mono">{r.code}</span> },
    { key: "title", header: "Курс", render: (r) => r.title },
    { key: "teacher", header: "Преподавател", render: (r) => r.teacher },
    { key: "semester", header: "Семестър", render: (r) => r.semester },
    { key: "status", header: "Статус", render: (r) => <StatusBadge status={r.status.toLowerCase()} /> },
    {
      key: "actions",
      header: <span className="uc-muted">Действие</span>,
      render: (r) =>
        r.status === "ENROLLED" ? (
          <ConfirmButton
            question="Да откажа ли записването?"
            busy={cancel.isPending && cancel.variables === r.id}
            onConfirm={() => cancel.mutate(r.id)}
          >
            Откажи
          </ConfirmButton>
        ) : null,
    },
  ];

  return (
    <>
      <PageHeader kicker="Студент" title="Моите записвания" />
      <Flash message={flash} onClose={() => setFlash(null)} />
      <QueryState query={query}>
        {(rows) => (
          <DataTable
            columns={columns}
            rows={rows}
            rowKey={(r) => r.id}
            empty={
              <EmptyState
                title="Още нямате записвания"
                action={
                  <Link to="/courses" className={buttonClass("primary")}>
                    Разгледайте курсовете
                  </Link>
                }
              />
            }
          />
        )}
      </QueryState>
    </>
  );
}
