import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import {
  Alert,
  Button,
  Card,
  Checkbox,
  DataTable,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  StatusBadge,
  TextArea,
  TextInput,
  type Column,
  type FlashMessage,
} from "@unicloud/ui";
import { useState, type FormEvent } from "react";
import { api, errorText, type Schemas } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { lines, str } from "../forms";

const columns: Column<Schemas["ModuleSummary"]>[] = [
  { key: "id", header: "Client ID", render: (r) => <span className="uc-mono">{r.client_id}</span> },
  { key: "name", header: "Име", render: (r) => r.name },
  { key: "uris", header: "Адреси за връщане", render: (r) => r.redirect_uris.join(", ") },
  { key: "dir", header: "Директория", render: (r) => <StatusBadge status={r.directory ? "active" : "disabled"} label={r.directory ? "Да" : "Не"} /> },
  { key: "created", header: "Създаден", render: (r) => r.created },
];

/** The secret is shown once, right after registration, and is never stored by the page. */
function SecretOnce({ reg, onDone }: { reg: Schemas["ClientRegistered"]; onDone: () => void }) {
  return (
    <Alert tone="info" onClose={onDone}>
      <p style={{ margin: "0 0 .5rem" }}>
        Модулът <strong>{reg.clientId}</strong> е регистриран. Запишете тайната сега. Тя няма да бъде показана отново.
      </p>
      <div className="uc-secret" aria-label="Тайна на клиента">{reg.clientSecret}</div>
    </Alert>
  );
}

export function AdminModulesPage() {
  const qc = useQueryClient();
  const [flash, setFlash] = useState<FlashMessage | null>(null);
  const [registered, setRegistered] = useState<Schemas["ClientRegistered"] | null>(null);
  const query = useQuery({ queryKey: keys.modules, queryFn: api.modules });

  const register = useMutation({
    mutationFn: api.registerModule,
    onSuccess: (r) => {
      setFlash(null);
      setRegistered(r);
      void qc.invalidateQueries({ queryKey: keys.modules });
    },
    onError: (e) => setFlash({ tone: "error", text: errorText(e) }),
  });

  const onSubmit = (e: FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const form = e.currentTarget;
    const fd = new FormData(form);
    const post = lines(fd, "post_logout");
    register.mutate(
      {
        clientId: str(fd, "client_id"),
        name: str(fd, "name"),
        redirectUris: lines(fd, "redirect_uris"),
        postLogoutRedirectUris: post.length ? post : undefined,
        directory: fd.get("directory") === "on",
      },
      { onSuccess: () => form.reset() },
    );
  };

  return (
    <>
      <PageHeader kicker="Администратор" title="Модули" lede="Регистрирайте модул на друг екип като OIDC клиент." />
      {registered && <SecretOnce reg={registered} onDone={() => setRegistered(null)} />}
      <Flash message={flash} onClose={() => setFlash(null)} />
      <QueryState query={query}>
        {(rows) => <DataTable columns={columns} rows={rows} rowKey={(r) => r.client_id} empty={<EmptyState title="Няма регистрирани модули" />} />}
      </QueryState>
      <Card title="Нов модул" style={{ marginTop: "1.4rem" }}>
        <form onSubmit={onSubmit}>
          <div className="uc-form-grid">
            <Field label="Client ID"><TextInput name="client_id" required /></Field>
            <Field label="Име"><TextInput name="name" required /></Field>
          </div>
          <Field label="Адреси за връщане" hint="По един на ред.">
            <TextArea name="redirect_uris" required />
          </Field>
          <Field label="Адреси след изход" hint="По един на ред. Не е задължително.">
            <TextArea name="post_logout" />
          </Field>
          <div className="uc-field">
            <Checkbox name="directory" label="Достъп до директорията" />
          </div>
          <Button type="submit" busy={register.isPending}>Регистрирай</Button>
        </form>
      </Card>
    </>
  );
}
