import { PageHeader } from "@unicloud/ui";
import { useQuery } from "@tanstack/react-query";
import { api } from "../api/client";
import { keys } from "../api/hooks";
import { QueryState } from "../components/QueryState";
import { UserDetails } from "../components/UserDetails";

export function ProfilePage() {
  const query = useQuery({ queryKey: keys.profile, queryFn: api.profile });
  return (
    <>
      <PageHeader kicker="Профил" title="Моите данни" lede="Данните идват от университетската директория." />
      <QueryState query={query}>{(u) => <UserDetails user={u} />}</QueryState>
    </>
  );
}
