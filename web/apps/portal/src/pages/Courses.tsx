import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Button,
  CourseCard,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  Select,
  TextInput,
  type FlashMessage,
} from "@unicloud/ui";
import { useState } from "react";
import { api, errorText, type Schemas } from "../api/client";
import { keys, useDebounced, useOptions } from "../api/hooks";
import { QueryState } from "../components/QueryState";

const SEARCH_DELAY_MS = 250;

function enrollFlash(r: Schemas["EnrollResult"]): FlashMessage {
  const ok = r.outcome === "ENROLLED";
  return { tone: ok ? "ok" : "error", text: r.message };
}

export function CoursesPage() {
  const [q, setQ] = useState("");
  const [faculty, setFaculty] = useState("");
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const debouncedQ = useDebounced(q.trim(), SEARCH_DELAY_MS);
  const qc = useQueryClient();
  const options = useOptions();

  const courses = useQuery({
    queryKey: keys.courses(debouncedQ, faculty),
    queryFn: () => api.courses(debouncedQ || undefined, faculty || undefined),
    placeholderData: (prev) => prev,
  });

  const enroll = useMutation({
    mutationFn: api.enroll,
    onSuccess: (r) => {
      setFlash(enrollFlash(r));
      void qc.invalidateQueries({ queryKey: ["courses"] });
      void qc.invalidateQueries({ queryKey: keys.enrollments });
    },
    onError: (e) => setFlash({ tone: "error", text: errorText(e) }),
  });

  const facultyOptions = [
    { value: "", label: "Всички факултети" },
    ...(options.data?.faculties ?? []).map((f) => ({ value: f.code, label: f.name })),
  ];

  return (
    <>
      <PageHeader
        kicker={options.data ? `Семестър ${options.data.semester}` : "Каталог"}
        title="Курсове"
        lede="Търсете по код или име на курса и се запишете с едно натискане."
      />
      <div className="uc-row" style={{ alignItems: "flex-end" }} role="search">
        <Field label="Търсене" className="uc-grow">
          <TextInput type="search" value={q} onChange={(e) => setQ(e.target.value)} placeholder="напр. CS101" />
        </Field>
        <Field label="Факултет">
          <Select value={faculty} onChange={(e) => setFaculty(e.target.value)} options={facultyOptions} />
        </Field>
      </div>
      <Flash message={flash} onClose={() => setFlash(null)} />
      <QueryState query={courses}>
        {(list) =>
          list.length === 0 ? (
            <EmptyState title="Няма намерени курсове">Променете търсенето или факултета.</EmptyState>
          ) : (
            <div className="uc-grid">
              {list.map((c) => {
                const enrolled = c.my_status === "ENROLLED";
                const full = c.enrolled >= c.capacity;
                const busy = enroll.isPending && enroll.variables === c.id;
                return (
                  <CourseCard
                    key={c.id}
                    code={c.code}
                    title={c.title}
                    status={c.my_status ? c.my_status.toLowerCase() : undefined}
                    meta={[c.faculty, c.teacher, `${c.ects} ECTS`]}
                    seats={{ taken: c.enrolled, capacity: c.capacity }}
                    actions={
                      <Button size="sm" busy={busy} disabled={enrolled || full} onClick={() => enroll.mutate(c.id)}>
                        {enrolled ? "Записан" : full ? "Няма места" : "Запиши се"}
                      </Button>
                    }
                  />
                );
              })}
            </div>
          )
        }
      </QueryState>
    </>
  );
}
