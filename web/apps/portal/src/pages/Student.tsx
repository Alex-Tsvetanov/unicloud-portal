import { PageHeader } from "@unicloud/ui";
import { useQuery } from "@tanstack/react-query";
import { useParams } from "react-router";
import { api } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { UserDetails } from "../components/UserDetails";

export function StudentPage() {
  const { sub = "" } = useParams();
  const query = useQuery({ queryKey: keys.student(sub), queryFn: () => api.student(sub), enabled: sub !== "" });
  return (
    <>
      <PageHeader kicker="Студент" title="Профил на студент" />
      <QueryState query={query}>{(u) => <UserDetails user={u} />}</QueryState>
    </>
  );
}
