import { QueryClient } from "@tanstack/react-query";
import { ApiError } from "./api/client";

/** Do not retry auth or client errors. The client already redirects on 401 and on a forced password change. */
export function createQueryClient(): QueryClient {
  return new QueryClient({
    defaultOptions: {
      queries: {
        staleTime: 30_000,
        refetchOnWindowFocus: false,
        retry: (count, err) => !(err instanceof ApiError && err.status < 500) && count < 2,
      },
      mutations: { retry: false },
    },
  });
}
