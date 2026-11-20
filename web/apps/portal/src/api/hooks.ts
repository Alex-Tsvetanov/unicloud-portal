import { useQuery } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { api, setCsrfToken } from "./client";

export const keys = {
  me: ["me"] as const,
  options: ["options"] as const,
  profile: ["profile"] as const,
  student: (sub: string) => ["student", sub] as const,
  courses: (q: string, faculty: string) => ["courses", q, faculty] as const,
  enrollments: ["enrollments"] as const,
  teach: ["teach"] as const,
  roster: (id: number) => ["roster", id] as const,
  faculty: ["faculty"] as const,
  inspectStudents: (q: string) => ["inspect-students", q] as const,
  inspectCourses: ["inspect-courses"] as const,
  staff: ["staff"] as const,
  modules: ["modules"] as const,
};

/** The signed-in user. Also stores the CSRF token for later writes. */
export function useMe() {
  return useQuery({
    queryKey: keys.me,
    queryFn: async () => {
      const me = await api.me();
      setCsrfToken(me.csrf);
      return me;
    },
    staleTime: 5 * 60_000,
  });
}

export function useOptions() {
  return useQuery({ queryKey: keys.options, queryFn: api.options, staleTime: 10 * 60_000 });
}

export function useDebounced<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);
  useEffect(() => {
    const id = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(id);
  }, [value, delayMs]);
  return debounced;
}
