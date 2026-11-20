import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Card,
  DataTable,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  Select,
  TextInput,
  type Column,
  type FlashMessage,
} from "@unicloud/ui";
import { useState, type FormEvent } from "react";
import { Link } from "react-router";
import { api, errorText, type Schemas } from "../api/client";
import { keys, useDebounced, useOptions } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { int, str } from "../forms";
import { STUDENT_STATUS } from "../labels";

type Row = Schemas["InspectStudent"];
type Status = Row["status"];

const STATUS_OPTIONS = (Object.keys(STUDENT_STATUS) as Status[]).map((s) => ({ value: s, label: STUDENT_STATUS[s] }));

export function InspectStudentsPage() {
  const qc = useQueryClient();
  const [q, setQ] = useState("");
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const debouncedQ = useDebounced(q.trim(), 250);
  const options = useOptions();
  const query = useQuery({
    queryKey: keys.inspectStudents(debouncedQ),
    queryFn: () => api.inspectStudents(debouncedQ || undefined),
    placeholderData: (prev) => prev,
  });

  const refresh = () => qc.invalidateQueries({ queryKey: ["inspect-students"] });
  const fail = (e: unknown) => setFlash({ tone: "error", text: errorText(e) });

  const setStatus = useMutation({
    mutationFn: ({ sub, status }: { sub: string; status: Status }) => api.setStudentStatus(sub, { status }),
    onSuccess: () => {
      setFlash({ tone: "ok", text: "Статусът е променен." });
      void refresh();
    },
    onError: fail,
  });

  const create = useMutation({
    mutationFn: api.createStudent,
    onSuccess: () => {
      setFlash({ tone: "ok", text: "Студентът е създаден." });
      void refresh();
    },
    onError: fail,
  });

  const onCreate = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    create.mutate(
      {
        email: str(fd, "email"),
        first_name: str(fd, "first_name"),
        last_name: str(fd, "last_name"),
        faculty_number: str(fd, "faculty_number"),
        specialty_id: int(fd, "specialty_id"),
        study_year: int(fd, "study_year"),
        password: str(fd, "password"),
      },
      { onSuccess: () => form.reset() },
    );
  };

  const columns: Column<Row>[] = [
    { key: "name", header: "Име", render: (r) => <Link to={`/students/${encodeURIComponent(r.sub)}`}>{r.name}</Link> },
    { key: "email", header: "Имейл", render: (r) => r.email },
    { key: "fn", header: "Фак. номер", render: (r) => <span className="uc-mono">{r.faculty_number}</span> },
    { key: "spec", header: "Специалност", render: (r) => r.specialty },
    { key: "year", header: "Курс", numeric: true, render: (r) => r.study_year },
    {
      key: "status",
      header: "Статус",
      render: (r) => (
        <Select
          aria-label={`Статус на ${r.name}`}
          value={r.status}
          disabled={setStatus.isPending}
          options={STATUS_OPTIONS}
          onChange={(e) => setStatus.mutate({ sub: r.sub, status: e.target.value as Status })}
        />
      ),
    },
  ];

  const specialties = options.data?.specialties ?? [];

  return (
    <>
      <PageHeader kicker="Инспектор" title="Студенти" />
      <Flash message={flash} onClose={() => setFlash(null)} />
      <Field label="Търсене">
        <TextInput type="search" value={q} onChange={(e) => setQ(e.target.value)} placeholder="Име, имейл или номер" />
      </Field>
      <QueryState query={query}>
        {(rows) => (
          <DataTable columns={columns} rows={rows} rowKey={(r) => r.sub} empty={<EmptyState title="Няма намерени студенти" />} />
        )}
      </QueryState>
      <Card title="Нов студент" style={{ marginTop: "1.4rem" }}>
        <form onSubmit={onCreate}>
          <div className="uc-form-grid">
            <Field label="Име"><TextInput name="first_name" required /></Field>
            <Field label="Фамилия"><TextInput name="last_name" required /></Field>
            <Field label="Имейл"><TextInput name="email" type="email" required /></Field>
            <Field label="Факултетен номер"><TextInput name="faculty_number" required /></Field>
            <Field label="Специалност">
              <Select name="specialty_id" required defaultValue="">
                <option value="" disabled>Изберете…</option>
                {specialties.map((s) => (
                  <option key={s.id} value={s.id}>{s.name} ({s.faculty_code})</option>
                ))}
              </Select>
            </Field>
            <Field label="Курс"><TextInput name="study_year" type="number" min={1} max={6} required /></Field>
            <Field label="Начална парола">
              <TextInput name="password" type="password" autoComplete="new-password" required />
            </Field>
          </div>
          <Button type="submit" busy={create.isPending}>Създай</Button>
        </form>
      </Card>
    </>
  );
}
