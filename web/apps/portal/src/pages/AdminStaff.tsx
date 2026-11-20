import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Card,
  DataTable,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  RoleBadge,
  Select,
  StatusBadge,
  TextInput,
  type Column,
  type FlashMessage,
} from "@unicloud/ui";
import { useState, type FormEvent } from "react";
import { api, errorText, type Schemas } from "../api/client";
import { keys, useOptions } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { str } from "../forms";
import { ROLE_LABEL, STAFF_ROLES } from "../labels";

type Row = Schemas["StaffMember"];

export function AdminStaffPage() {
  const qc = useQueryClient();
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const options = useOptions();
  const query = useQuery({ queryKey: keys.staff, queryFn: api.staff });

  const refresh = () => qc.invalidateQueries({ queryKey: keys.staff });
  const fail = (e: unknown) => setFlash({ tone: "error", text: errorText(e) });

  const toggle = useMutation({
    mutationFn: api.toggleStaff,
    onSuccess: (r) => {
      setFlash({ tone: "ok", text: r.enabled ? "Профилът е включен." : "Профилът е изключен." });
      void refresh();
    },
    onError: fail,
  });

  const create = useMutation({
    mutationFn: api.createStaff,
    onSuccess: () => {
      setFlash({ tone: "ok", text: "Служителят е създаден." });
      void refresh();
    },
    onError: fail,
  });

  const onCreate = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    const faculty = str(fd, "faculty_code");
    create.mutate(
      {
        email: str(fd, "email"),
        first_name: str(fd, "first_name"),
        last_name: str(fd, "last_name"),
        role: str(fd, "role") as Schemas["Role"],
        faculty_code: faculty || null,
        password: str(fd, "password"),
      },
      { onSuccess: () => form.reset() },
    );
  };

  const columns: Column<Row>[] = [
    { key: "name", header: "Име", render: (r) => r.name },
    { key: "email", header: "Имейл", render: (r) => r.email },
    { key: "role", header: "Роля", render: (r) => <RoleBadge role={r.role} label={ROLE_LABEL[r.role]} /> },
    { key: "faculty", header: "Факултет", render: (r) => r.faculty ?? <span className="uc-muted">няма</span> },
    { key: "enabled", header: "Състояние", render: (r) => <StatusBadge status={r.enabled ? "active" : "disabled"} /> },
    {
      key: "actions",
      header: <span className="uc-muted">Действие</span>,
      render: (r) => (
        <Button
          size="sm"
          variant={r.enabled ? "danger" : "secondary"}
          busy={toggle.isPending && toggle.variables === r.sub}
          onClick={() => toggle.mutate(r.sub)}
        >
          {r.enabled ? "Изключи" : "Включи"}
        </Button>
      ),
    },
  ];

  return (
    <>
      <PageHeader kicker="Администратор" title="Служители" />
      <Flash message={flash} onClose={() => setFlash(null)} />
      <QueryState query={query}>
        {(rows) => <DataTable columns={columns} rows={rows} rowKey={(r) => r.sub} empty={<EmptyState title="Няма служители" />} />}
      </QueryState>
      <Card title="Нов служител" style={{ marginTop: "1.4rem" }}>
        <form onSubmit={onCreate}>
          <div className="uc-form-grid">
            <Field label="Име"><TextInput name="first_name" required /></Field>
            <Field label="Фамилия"><TextInput name="last_name" required /></Field>
            <Field label="Имейл"><TextInput name="email" type="email" required /></Field>
            <Field label="Роля">
              <Select name="role" required options={STAFF_ROLES.map((r) => ({ value: r, label: ROLE_LABEL[r] }))} />
            </Field>
            <Field label="Факултет">
              <Select name="faculty_code" defaultValue="">
                <option value="">Без факултет</option>
                {(options.data?.faculties ?? []).map((f) => (
                  <option key={f.code} value={f.code}>{f.name}</option>
                ))}
              </Select>
            </Field>
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
