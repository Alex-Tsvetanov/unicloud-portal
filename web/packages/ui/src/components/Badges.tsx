import { cx } from "../cx";

type Tone = "ok" | "warn" | "err" | "info" | "neutral" | "accent";

interface Known {
  tone: Tone;
  label: string;
}

const STATUSES: Record<string, Known> = {
  enrolled: { tone: "ok", label: "Записан" },
  active: { tone: "ok", label: "Активен" },
  approved: { tone: "ok", label: "Одобрен" },
  open: { tone: "ok", label: "Отворен" },
  pending: { tone: "warn", label: "Чака" },
  waitlisted: { tone: "warn", label: "Чакащ" },
  cancelled: { tone: "neutral", label: "Отказан" },
  canceled: { tone: "neutral", label: "Отказан" },
  closed: { tone: "neutral", label: "Затворен" },
  disabled: { tone: "neutral", label: "Изключен" },
  rejected: { tone: "err", label: "Отхвърлен" },
  full: { tone: "err", label: "Пълен" },
};

const ROLES: Record<string, Known> = {
  student: { tone: "info", label: "Студент" },
  teacher: { tone: "accent", label: "Преподавател" },
  admin: { tone: "err", label: "Администратор" },
  dean: { tone: "warn", label: "Декан" },
  faculty_admin: { tone: "warn", label: "Факултетен администратор" },
  inspector: { tone: "neutral", label: "Инспектор" },
};

export interface StatusBadgeProps {
  status: string;
  /** Overrides the built-in label. */
  label?: string;
  className?: string;
}

/** Known statuses get a colour and a Bulgarian label. Unknown ones render as-is, in a neutral colour. */
export function StatusBadge({ status, label, className }: StatusBadgeProps) {
  const known = STATUSES[status.toLowerCase()];
  return (
    <span className={cx("uc-badge", `uc-badge-${known?.tone ?? "neutral"}`, className)} data-status={status}>
      {label ?? known?.label ?? status}
    </span>
  );
}

export interface RoleBadgeProps {
  role: string;
  label?: string;
  className?: string;
}

/** Known roles get a colour and a Bulgarian label. Unknown ones render as-is, in a neutral colour. */
export function RoleBadge({ role, label, className }: RoleBadgeProps) {
  const known = ROLES[role.toLowerCase()];
  return (
    <span className={cx("uc-badge", `uc-badge-${known?.tone ?? "neutral"}`, className)} data-role={role}>
      {label ?? known?.label ?? role}
    </span>
  );
}
