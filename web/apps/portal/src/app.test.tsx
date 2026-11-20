import { QueryClientProvider } from "@tanstack/react-query";
import { render, screen, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router";
import { App } from "./App";
import { ApiError, api, browser, request, setCsrfToken, type Schemas } from "./api/client";
import { createQueryClient } from "./queryClient";

const ME: Schemas["Me"] = {
  user: {
    sub: "u1",
    email: "maria@tu-sofia.bg",
    given_name: "Мария",
    family_name: "Петрова",
    name: "Мария Петрова",
    role: "STUDENT",
    role_label: "Студент",
    faculty: { code: "FKSU", name: "Факултет по компютърни системи и управление" },
    permissions: ["ENROLLMENT_REQUEST_CREATE"],
  },
  csrf: "tok-123",
  semester: "2026-W",
  nav: [
    { path: "/home", label: "Начало" },
    { path: "/courses", label: "Курсове" },
    { path: "/enrollments", label: "Записвания" },
  ],
};

const COURSES: Schemas["CatalogOffering"][] = [
  {
    id: 7,
    code: "CS101",
    title: "Програмиране",
    ects: 5,
    faculty: "ФКСУ",
    teacher: "доц. Иванов",
    enrolled: 3,
    capacity: 30,
    my_status: null,
  },
];

type Handler = (url: string, init: RequestInit) => { status: number; body?: unknown };

function json(status: number, body?: unknown): Response {
  return new Response(body === undefined ? null : JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/json" },
  });
}

function mockFetch(handler: Handler) {
  const fn = vi.fn(async (input: RequestInfo | URL, init: RequestInit = {}) => {
    const { status, body } = handler(String(input), init);
    return json(status, body);
  });
  vi.stubGlobal("fetch", fn);
  return fn;
}

function headerOf(init: RequestInit | undefined, name: string): string | undefined {
  return (init?.headers as Record<string, string> | undefined)?.[name];
}

beforeEach(() => {
  setCsrfToken(null);
  vi.spyOn(browser, "currentPath").mockReturnValue("/app/courses?q=x");
});

afterEach(() => {
  vi.unstubAllGlobals();
  vi.restoreAllMocks();
});

describe("api client", () => {
  it("fetches /me for the CSRF token and sends it on a write", async () => {
    const fetchFn = mockFetch((url) => {
      if (url.endsWith("/me")) return { status: 200, body: ME };
      return { status: 200, body: { outcome: "ENROLLED", message: "Записахте се." } };
    });
    const result = await api.enroll(7);
    expect(result.outcome).toBe("ENROLLED");
    const [meCall, postCall] = fetchFn.mock.calls;
    expect(String(meCall[0])).toBe("/api/v1/portal/me");
    expect(String(postCall[0])).toBe("/api/v1/portal/courses/7/enroll");
    expect(postCall[1]?.method).toBe("POST");
    expect(postCall[1]?.credentials).toBe("same-origin");
    expect(headerOf(postCall[1], "X-CSRF-Token")).toBe("tok-123");
  });

  it("does not send the CSRF header on GET", async () => {
    const fetchFn = mockFetch(() => ({ status: 200, body: [] }));
    await api.enrollments();
    expect(fetchFn).toHaveBeenCalledOnce();
    expect(headerOf(fetchFn.mock.calls[0][1], "X-CSRF-Token")).toBeUndefined();
  });

  it("returns the EnrollResult body on 409", async () => {
    setCsrfToken("tok");
    mockFetch(() => ({ status: 409, body: { outcome: "FULL", message: "Няма свободни места." } }));
    await expect(api.enroll(7)).resolves.toEqual({ outcome: "FULL", message: "Няма свободни места." });
  });

  it("does a full page load to /login with next on 401", async () => {
    const assign = vi.spyOn(browser, "assign").mockImplementation(() => {});
    mockFetch(() => ({ status: 401, body: { error: "unauthenticated", message: "Влезте отново." } }));
    await expect(request("GET", "/profile")).rejects.toBeInstanceOf(ApiError);
    expect(assign).toHaveBeenCalledWith("/login?next=%2Fapp%2Fcourses%3Fq%3Dx");
  });

  it("sends the browser to /password on a forced password change", async () => {
    const assign = vi.spyOn(browser, "assign").mockImplementation(() => {});
    mockFetch(() => ({ status: 403, body: { error: "password_change_required", message: "Сменете паролата." } }));
    await expect(api.profile()).rejects.toThrow("Сменете паролата.");
    expect(assign).toHaveBeenCalledWith("/password");
  });

  it("keeps the server's message on other errors and does not redirect", async () => {
    const assign = vi.spyOn(browser, "assign").mockImplementation(() => {});
    setCsrfToken("tok");
    mockFetch(() => ({ status: 409, body: { error: "conflict", message: "Записването вече е отхвърлено." } }));
    await expect(api.reject(5)).rejects.toThrow("Записването вече е отхвърлено.");
    expect(assign).not.toHaveBeenCalled();
  });
});

function renderAt(path: string) {
  return render(
    <QueryClientProvider client={createQueryClient()}>
      <MemoryRouter initialEntries={[path]}>
        <App />
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe("pages", () => {
  it("renders the courses page, enrolls and shows the server message", async () => {
    const fetchFn = mockFetch((url, init) => {
      if (url.endsWith("/me")) return { status: 200, body: ME };
      if (url.endsWith("/options")) return { status: 200, body: { faculties: [], specialties: [], semester: "2026-W" } };
      if (url.includes("/courses/7/enroll") && init.method === "POST") {
        return { status: 200, body: { outcome: "ENROLLED", message: "Записахте се за CS101." } };
      }
      if (url.includes("/courses")) return { status: 200, body: COURSES };
      return { status: 404, body: { error: "not_found", message: "Няма такова нещо." } };
    });

    renderAt("/courses");
    expect(await screen.findByRole("heading", { level: 3, name: "Програмиране" })).toBeInTheDocument();
    expect(screen.getByRole("link", { name: "Курсове" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByText("3 / 30 места")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Запиши се" }));
    expect(await screen.findByText("Записахте се за CS101.")).toBeInTheDocument();
    const post = fetchFn.mock.calls.find(([, init]) => init?.method === "POST");
    expect(headerOf(post?.[1], "X-CSRF-Token")).toBe("tok-123");
  });

  it("reads the catalogue filter from the URL", async () => {
    const fetchFn = mockFetch((url) => {
      if (url.endsWith("/me")) return { status: 200, body: ME };
      if (url.endsWith("/options")) return { status: 200, body: { faculties: [], specialties: [], semester: "2026-W" } };
      return { status: 200, body: COURSES };
    });
    renderAt("/courses?q=CS1&faculty=FCST");
    expect(await screen.findByRole("heading", { level: 3, name: "Програмиране" })).toBeInTheDocument();
    expect(screen.getByRole("searchbox")).toHaveValue("CS1");
    const call = fetchFn.mock.calls.map(([u]) => String(u)).find((u) => u.includes("/courses?"));
    expect(call).toContain("q=CS1");
    expect(call).toContain("faculty=FCST");
  });

  it("redirects / to /home and renders a sign-out form with the CSRF field", async () => {
    mockFetch((url) => (url.endsWith("/me") ? { status: 200, body: ME } : { status: 200, body: [] }));
    const { container } = renderAt("/");
    expect(await screen.findByRole("heading", { level: 1, name: "Здравейте, Мария" })).toBeInTheDocument();
    const form = container.querySelector('form[action="/logout"]');
    expect(form).toHaveAttribute("method", "post");
    expect(form?.querySelector('input[name="csrf"]')).toHaveValue("tok-123");
  });

  it("shows a 404 page for unknown paths", async () => {
    mockFetch(() => ({ status: 200, body: ME }));
    renderAt("/no/such/page");
    expect(await screen.findByRole("heading", { name: "Страницата не е намерена" })).toBeInTheDocument();
  });

  it("cancels an enrollment only after confirming", async () => {
    let cancelled = false;
    const fetchFn = mockFetch((url, init) => {
      if (url.endsWith("/me")) return { status: 200, body: ME };
      if (url.endsWith("/enrollments/11/cancel") && init.method === "POST") {
        cancelled = true;
        return { status: 200 };
      }
      return {
        status: 200,
        body: [
          {
            id: 11,
            code: "CS101",
            title: "Програмиране",
            semester: "2026-W",
            teacher: "доц. Иванов",
            status: cancelled ? "CANCELLED" : "ENROLLED",
          },
        ],
      };
    });
    renderAt("/enrollments");
    await userEvent.click(await screen.findByRole("button", { name: "Откажи" }));
    expect(fetchFn.mock.calls.some(([, init]) => init?.method === "POST")).toBe(false);
    await userEvent.click(screen.getByRole("button", { name: "Да" }));
    expect(await screen.findByText("Записването е отказано.")).toBeInTheDocument();
    await waitFor(() => expect(screen.getByText("Отказан")).toBeInTheDocument());
  });
});
