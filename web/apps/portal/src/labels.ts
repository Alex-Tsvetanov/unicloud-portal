import type { Schemas } from "./api/client";

export const STUDENT_STATUS: Record<Schemas["InspectStudent"]["status"], string> = {
  ACTIVE: "Активен",
  INTERRUPTED: "Прекъснал",
  GRADUATED: "Завършил",
};

export const ROLE_LABEL: Record<Schemas["Role"], string> = {
  STUDENT: "Студент",
  TEACHER: "Преподавател",
  DEAN: "Декан",
  INSPECTOR: "Инспектор",
  ADMIN: "Администратор",
};

export const STAFF_ROLES: Schemas["Role"][] = ["TEACHER", "DEAN", "INSPECTOR", "ADMIN"];

/** Status words the shared StatusBadge knows, mapped from API enums. Unknown values pass through. */
export function badgeStatus(s: string): string {
  return s.toLowerCase();
}
