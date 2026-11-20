import type { components } from "./schema";

export type Schemas = components["schemas"];
export type PortalErrorBody = Schemas["PortalError"];

const API = "/api/v1/portal";
const APP_BASE = "/app";

/** Full page loads go through this object so tests can replace them. */
export const browser = {
  assign(url: string): void {
    window.location.assign(url);
  },
  currentPath(): string {
    return window.location.pathname + window.location.search;
  },
};

/** An error from the portal API. `message` is the server's Bulgarian text when it sent one. */
export class ApiError extends Error {
  readonly status: number;
  readonly code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
  }
}

const FALLBACK_MESSAGE = "Възникна грешка. Опитайте отново.";

let csrfToken: string | null = null;

export function setCsrfToken(token: string | null): void {
  csrfToken = token;
}

/** Where the login page should send the user back to. Always a path under /app. */
export function loginUrl(): string {
  const here = browser.currentPath();
  const next = here.startsWith(APP_BASE) ? here : APP_BASE + here;
  return `/login?next=${encodeURIComponent(next)}`;
}

async function readJson(res: Response): Promise<unknown> {
  const text = await res.text();
  if (!text) return undefined;
  try {
    return JSON.parse(text);
  } catch {
    return undefined;
  }
}

function isPortalError(body: unknown): body is PortalErrorBody {
  return typeof body === "object" && body !== null && "error" in body && "message" in body;
}

async function ensureCsrf(): Promise<string> {
  if (csrfToken) return csrfToken;
  const me = await request<Schemas["Me"]>("GET", "/me");
  csrfToken = me.csrf;
  return me.csrf;
}

export interface RequestOptions {
  body?: unknown;
  /** Extra statuses whose JSON body is returned as data, for example 409 on enroll. */
  acceptStatuses?: number[];
}

export async function request<T>(method: string, path: string, opts: RequestOptions = {}): Promise<T> {
  const headers: Record<string, string> = { Accept: "application/json" };
  if (method !== "GET" && method !== "HEAD") {
    headers["X-CSRF-Token"] = await ensureCsrf();
  }
  if (opts.body !== undefined) headers["Content-Type"] = "application/json";

  const res = await fetch(API + path, {
    method,
    headers,
    credentials: "same-origin",
    body: opts.body === undefined ? undefined : JSON.stringify(opts.body),
  });
  const body = await readJson(res);

  if (res.ok || opts.acceptStatuses?.includes(res.status)) return body as T;

  if (res.status === 401) {
    browser.assign(loginUrl());
  } else if (res.status === 403 && isPortalError(body) && body.error === "password_change_required") {
    browser.assign("/password");
  }
  if (isPortalError(body)) throw new ApiError(res.status, body.error, body.message || FALLBACK_MESSAGE);
  throw new ApiError(res.status, "server_error", FALLBACK_MESSAGE);
}

/** The text to show for any thrown value. */
export function errorText(e: unknown): string {
  if (e instanceof ApiError) return e.message;
  return FALLBACK_MESSAGE;
}

function qs(params: Record<string, string | undefined>): string {
  const p = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) if (v) p.set(k, v);
  const s = p.toString();
  return s ? `?${s}` : "";
}

const enc = encodeURIComponent;

export const api = {
  me: () => request<Schemas["Me"]>("GET", "/me"),
  options: () => request<Schemas["Options"]>("GET", "/options"),
  profile: () => request<Schemas["DirectoryUser"]>("GET", "/profile"),
  student: (sub: string) => request<Schemas["DirectoryUser"]>("GET", `/students/${enc(sub)}`),
  courses: (q?: string, faculty?: string) =>
    request<Schemas["CatalogOffering"][]>("GET", `/courses${qs({ q, faculty })}`),
  enroll: (id: number) =>
    request<Schemas["EnrollResult"]>("POST", `/courses/${id}/enroll`, { acceptStatuses: [409] }),
  enrollments: () => request<Schemas["MyEnrollment"][]>("GET", "/enrollments"),
  cancelEnrollment: (id: number) => request<void>("POST", `/enrollments/${id}/cancel`),
  teachOfferings: () => request<Schemas["TeachOffering"][]>("GET", "/teach/offerings"),
  roster: (id: number) => request<Schemas["OfferingRoster"]>("GET", `/teach/offerings/${id}`),
  reject: (enrollmentId: number) => request<void>("POST", `/teach/enrollments/${enrollmentId}/reject`),
  faculty: () => request<Schemas["FacultyOverview"]>("GET", "/faculty"),
  inspectStudents: (q?: string) => request<Schemas["InspectStudent"][]>("GET", `/inspect/students${qs({ q })}`),
  createStudent: (body: Schemas["NewStudent"]) =>
    request<Schemas["Created"]>("POST", "/inspect/students", { body }),
  setStudentStatus: (sub: string, body: Schemas["StatusChange"]) =>
    request<void>("PUT", `/inspect/students/${enc(sub)}/status`, { body }),
  inspectCourses: () => request<Schemas["InspectCourses"]>("GET", "/inspect/courses"),
  createCourse: (body: Schemas["NewCourse"]) => request<void>("POST", "/inspect/courses", { body }),
  createOffering: (body: Schemas["NewOffering"]) => request<void>("POST", "/inspect/offerings", { body }),
  toggleOffering: (id: number) => request<Schemas["Toggled"]>("POST", `/inspect/offerings/${id}/toggle`),
  staff: () => request<Schemas["StaffMember"][]>("GET", "/admin/staff"),
  createStaff: (body: Schemas["NewStaff"]) => request<Schemas["Created"]>("POST", "/admin/staff", { body }),
  toggleStaff: (sub: string) => request<Schemas["Toggled"]>("POST", `/admin/staff/${enc(sub)}/toggle`),
  modules: () => request<Schemas["ModuleSummary"][]>("GET", "/admin/modules"),
  registerModule: (body: Schemas["ClientRegistration"]) =>
    request<Schemas["ClientRegistered"]>("POST", "/admin/modules", { body }),
};
