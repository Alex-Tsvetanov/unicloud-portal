import type { UseQueryResult } from "@tanstack/react-query";
import { Alert } from "@unicloud/ui";
import type { ReactNode } from "react";
import { errorText } from "../api/client";

interface QueryStateProps<T> {
  query: UseQueryResult<T>;
  children: (data: T) => ReactNode;
}

/** Shows a loading line, the server's error message, or the data. */
export function QueryState<T>({ query, children }: QueryStateProps<T>) {
  if (query.isPending) return <p className="uc-muted">Зареждане…</p>;
  if (query.isError) return <Alert tone="error">{errorText(query.error)}</Alert>;
  return <>{children(query.data)}</>;
}
