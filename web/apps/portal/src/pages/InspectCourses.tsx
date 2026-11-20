import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  Card,
  DataTable,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  SeatsMeter,
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
import { int, str } from "../forms";

const courseCols: Column<Schemas["InspectCourse"]>[] = [
  { key: "code", header: "Код", render: (r) => <span className="uc-mono">{r.code}</span> },
  { key: "title", header: "Име", render: (r) => r.title },
  { key: "ects", header: "ECTS", numeric: true, render: (r) => r.ects },
  { key: "faculty", header: "Факултет", render: (r) => r.faculty },
];

export function InspectCoursesPage() {
  const qc = useQueryClient();
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const options = useOptions();
  const query = useQuery({ queryKey: keys.inspectCourses, queryFn: api.inspectCourses });

  const refresh = () => qc.invalidateQueries({ queryKey: keys.inspectCourses });
  const done = (text: string) => () => {
    setFlash({ tone: "ok", text });
    void refresh();
  };
  const fail = (e: unknown) => setFlash({ tone: "error", text: errorText(e) });

  const toggle = useMutation({
    mutationFn: api.toggleOffering,
    onSuccess: (r) => done(r.enabled ? "Записването е отворено." : "Записването е затворено.")(),
    onError: fail,
  });
  const createCourse = useMutation({ mutationFn: api.createCourse, onSuccess: done("Курсът е създаден."), onError: fail });
  const createOffering = useMutation({
    mutationFn: api.createOffering,
    onSuccess: done("Курсът е обявен за семестъра."),
    onError: fail,
  });

  const onCourse = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    createCourse.mutate(
      {
        code: str(fd, "code"),
        title: str(fd, "title"),
        ects_credits: int(fd, "ects_credits"),
        faculty_code: str(fd, "faculty_code"),
      },
      { onSuccess: () => form.reset() },
    );
  };

  const onOffering = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    createOffering.mutate(
      {
        course_code: str(fd, "course_code"),
        semester: str(fd, "semester"),
        teacher_sub: str(fd, "teacher_sub"),
        capacity: int(fd, "capacity"),
      },
      { onSuccess: () => form.reset() },
    );
  };

  const offeringCols: Column<Schemas["InspectOffering"]>[] = [
    { key: "code", header: "Код", render: (r) => <span className="uc-mono">{r.code}</span> },
    { key: "semester", header: "Семестър", render: (r) => r.semester },
    { key: "teacher", header: "Преподавател", render: (r) => r.teacher },
    { key: "seats", header: "Места", render: (r) => <SeatsMeter taken={r.enrolled} capacity={r.capacity} /> },
    { key: "open", header: "Записване", render: (r) => <StatusBadge status={r.is_open ? "open" : "closed"} /> },
    {
      key: "actions",
      header: <span className="uc-muted">Действие</span>,
      render: (r) => (
        <Button
          size="sm"
          variant="secondary"
          busy={toggle.isPending && toggle.variables === r.id}
          onClick={() => toggle.mutate(r.id)}
        >
          {r.is_open ? "Затвори" : "Отвори"}
        </Button>
      ),
    },
  ];

  const faculties = options.data?.faculties ?? [];

  return (
    <>
      <PageHeader kicker="Инспектор" title="Курсове и обявявания" />
      <Flash message={flash} onClose={() => setFlash(null)} />
      <QueryState query={query}>
        {(data) => (
          <>
            <Card title="Обявени курсове">
              <DataTable columns={offeringCols} rows={data.offerings} rowKey={(r) => r.id} empty={<EmptyState title="Няма обявени курсове" />} />
            </Card>
            <Card title="Курсове">
              <DataTable columns={courseCols} rows={data.courses} rowKey={(r) => r.code} empty={<EmptyState title="Няма курсове" />} />
            </Card>
            <div className="uc-grid">
              <Card title="Нов курс">
                <form onSubmit={onCourse}>
                  <Field label="Код"><TextInput name="code" required /></Field>
                  <Field label="Име"><TextInput name="title" required /></Field>
                  <Field label="ECTS кредити"><TextInput name="ects_credits" type="number" min={1} required /></Field>
                  <Field label="Факултет">
                    <Select name="faculty_code" required defaultValue="">
                      <option value="" disabled>Изберете…</option>
                      {faculties.map((f) => (
                        <option key={f.code} value={f.code}>{f.name}</option>
                      ))}
                    </Select>
                  </Field>
                  <Button type="submit" busy={createCourse.isPending}>Създай курс</Button>
                </form>
              </Card>
              <Card title="Обяви курс">
                <form onSubmit={onOffering}>
                  <Field label="Курс">
                    <Select name="course_code" required defaultValue="">
                      <option value="" disabled>Изберете…</option>
                      {data.courses.map((c) => (
                        <option key={c.code} value={c.code}>{c.code} {c.title}</option>
                      ))}
                    </Select>
                  </Field>
                  <Field label="Семестър">
                    <TextInput name="semester" required defaultValue={options.data?.semester ?? ""} key={options.data?.semester} />
                  </Field>
                  <Field label="Преподавател">
                    <Select name="teacher_sub" required defaultValue="">
                      <option value="" disabled>Изберете…</option>
                      {data.teachers.map((t) => (
                        <option key={t.sub} value={t.sub}>{t.name}{t.faculty ? ` (${t.faculty})` : ""}</option>
                      ))}
                    </Select>
                  </Field>
                  <Field label="Места"><TextInput name="capacity" type="number" min={1} required /></Field>
                  <Button type="submit" busy={createOffering.isPending}>Обяви</Button>
                </form>
              </Card>
            </div>
          </>
        )}
      </QueryState>
    </>
  );
}
