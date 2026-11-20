import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import {
  Alert,
  Button,
  Card,
  Checkbox,
  ConfirmButton,
  CourseCard,
  DataTable,
  EmptyState,
  Field,
  Flash,
  PageHeader,
  RoleBadge,
  SeatsMeter,
  Select,
  Stat,
  StatusBadge,
  StudentCard,
  TextArea,
  TextInput,
  TopBar,
  buttonClass,
  initials,
} from "./index";

describe("TopBar", () => {
  it("renders nav items as plain links by default and marks the active one", () => {
    render(
      <TopBar
        items={[
          { label: "Начало", href: "/home", active: true },
          { label: "Курсове", href: "/courses" },
        ]}
        right={<span>Иван</span>}
      />,
    );
    expect(screen.getByRole("link", { name: "Начало" })).toHaveAttribute("aria-current", "page");
    expect(screen.getByRole("link", { name: "Курсове" })).toHaveAttribute("href", "/courses");
    expect(screen.getByText("Иван")).toBeInTheDocument();
  });

  it("uses renderLink when given", () => {
    render(
      <TopBar
        items={[{ label: "Профил", href: "/profile" }]}
        renderLink={({ item, children, className }) => (
          <button type="button" className={className} data-href={item.href}>
            {children}
          </button>
        )}
      />,
    );
    expect(screen.getByRole("button", { name: "Профил" })).toHaveAttribute("data-href", "/profile");
  });
});

describe("Button", () => {
  it("applies variant and size classes and defaults to type=button", () => {
    render(<Button variant="danger" size="sm">Изтрий</Button>);
    const btn = screen.getByRole("button", { name: "Изтрий" });
    expect(btn).toHaveClass("uc-btn", "uc-btn-danger", "uc-btn-sm");
    expect(btn).toHaveAttribute("type", "button");
  });

  it("is disabled while busy", () => {
    render(<Button busy>Запиши</Button>);
    expect(screen.getByRole("button")).toBeDisabled();
  });

  it("buttonClass omits the md size", () => {
    expect(buttonClass("secondary")).toBe("uc-btn uc-btn-secondary");
  });
});

describe("Field", () => {
  it("links the label, hint and error to the control", () => {
    render(
      <Field label="Имейл" hint="Университетски адрес" error="Задължително">
        <TextInput />
      </Field>,
    );
    const input = screen.getByLabelText("Имейл");
    expect(input).toHaveAttribute("aria-invalid", "true");
    expect(input.getAttribute("aria-describedby")).toMatch(/hint/);
    expect(input).toHaveAccessibleDescription("Университетски адрес Задължително");
  });

  it("renders Select options, TextArea and Checkbox", () => {
    render(
      <>
        <Select aria-label="Факултет" options={[{ value: "fksu", label: "ФКСУ" }]} />
        <TextArea aria-label="Бележка" />
        <Checkbox label="Активен" />
      </>,
    );
    expect(screen.getByRole("option", { name: "ФКСУ" })).toBeInTheDocument();
    expect(screen.getByRole("textbox", { name: "Бележка" })).toHaveClass("uc-textarea");
    expect(screen.getByRole("checkbox", { name: "Активен" })).toBeInTheDocument();
  });
});

describe("Alert and Flash", () => {
  it("uses role=alert for errors and closes", async () => {
    const onClose = vi.fn();
    render(<Alert tone="error" onClose={onClose}>Грешка</Alert>);
    expect(screen.getByRole("alert")).toHaveTextContent("Грешка");
    await userEvent.click(screen.getByRole("button", { name: "Затвори" }));
    expect(onClose).toHaveBeenCalledOnce();
  });

  it("Flash renders nothing without a message", () => {
    const { container } = render(<Flash message={null} />);
    expect(container).toBeEmptyDOMElement();
  });

  it("Flash renders a status message", () => {
    render(<Flash message={{ tone: "ok", text: "Записахте се." }} />);
    expect(screen.getByRole("status")).toHaveTextContent("Записахте се.");
  });
});

describe("badges", () => {
  it("translates known statuses and roles", () => {
    render(
      <>
        <StatusBadge status="enrolled" />
        <RoleBadge role="teacher" />
      </>,
    );
    expect(screen.getByText("Записан")).toHaveClass("uc-badge-ok");
    expect(screen.getByText("Преподавател")).toHaveClass("uc-badge-accent");
  });

  it("renders unknown statuses and roles as-is in a neutral tone", () => {
    render(
      <>
        <StatusBadge status="archived_by_team7" />
        <RoleBadge role="librarian" />
      </>,
    );
    expect(screen.getByText("archived_by_team7")).toHaveClass("uc-badge-neutral");
    expect(screen.getByText("librarian")).toHaveClass("uc-badge-neutral");
  });
});

describe("SeatsMeter", () => {
  it("shows the count and a level", () => {
    const { container } = render(<SeatsMeter taken={9} capacity={10} />);
    expect(screen.getByText("9 / 10 места")).toBeInTheDocument();
    expect(container.firstChild).toHaveAttribute("data-level", "warn");
    expect(screen.getByRole("meter")).toHaveAttribute("aria-valuenow", "9");
  });

  it("is full at capacity and with zero capacity", () => {
    const { container, rerender } = render(<SeatsMeter taken={3} capacity={3} />);
    expect(container.firstChild).toHaveAttribute("data-level", "full");
    rerender(<SeatsMeter taken={0} capacity={0} />);
    expect(container.firstChild).toHaveAttribute("data-level", "full");
  });
});

describe("DataTable", () => {
  const columns = [
    { key: "name", header: "Име", render: (r: { id: number; name: string }) => r.name },
    { key: "id", header: "№", numeric: true, render: (r: { id: number; name: string }) => r.id },
  ];

  it("renders headers and rows", () => {
    render(<DataTable columns={columns} rows={[{ id: 1, name: "Мария" }]} rowKey={(r) => r.id} />);
    expect(screen.getByRole("columnheader", { name: "Име" })).toBeInTheDocument();
    expect(screen.getByRole("cell", { name: "Мария" })).toBeInTheDocument();
    expect(screen.getByRole("cell", { name: "1" })).toHaveClass("uc-num");
  });

  it("renders the empty slot when there are no rows", () => {
    render(<DataTable columns={columns} rows={[]} rowKey={(r) => r.id} empty={<EmptyState title="Няма записи" />} />);
    expect(screen.queryByRole("table")).not.toBeInTheDocument();
    expect(screen.getByText("Няма записи")).toBeInTheDocument();
  });
});

describe("layout pieces", () => {
  it("PageHeader, Card and Stat render their content", () => {
    render(
      <>
        <PageHeader kicker="Студент" title="Курсове" lede="Изберете курс" actions={<Button>Ново</Button>} />
        <Card title="Профил" aside={<span>ok</span>}>Съдържание</Card>
        <Stat label="Курсове" value={4} hint="този семестър" />
      </>,
    );
    expect(screen.getByRole("heading", { level: 1, name: "Курсове" })).toBeInTheDocument();
    expect(screen.getByRole("heading", { level: 2, name: "Профил" })).toBeInTheDocument();
    expect(screen.getByText("4")).toHaveClass("uc-stat-value");
  });

  it("CourseCard shows code, meta, seats and actions", () => {
    render(
      <CourseCard
        code="CS101"
        title="Програмиране"
        meta={["ФКСУ", "5 кредита"]}
        seats={{ taken: 2, capacity: 30 }}
        status="open"
        actions={<Button size="sm">Запиши се</Button>}
      />,
    );
    expect(screen.getByText("CS101")).toBeInTheDocument();
    expect(screen.getByText("Отворен")).toBeInTheDocument();
    expect(screen.getByText("2 / 30 места")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Запиши се" })).toBeInTheDocument();
  });

  it("StudentCard shows initials", () => {
    render(<StudentCard name="Мария Петрова Иванова" meta={["121220001"]} />);
    expect(screen.getByText("МИ")).toBeInTheDocument();
    expect(initials("")).toBe("?");
    expect(initials("ana")).toBe("A");
  });
});

describe("ConfirmButton", () => {
  it("asks first and only confirms on the second click", async () => {
    const onConfirm = vi.fn();
    render(<ConfirmButton onConfirm={onConfirm} question="Да откажа ли?">Откажи</ConfirmButton>);
    await userEvent.click(screen.getByRole("button", { name: "Откажи" }));
    expect(onConfirm).not.toHaveBeenCalled();
    expect(screen.getByText("Да откажа ли?")).toBeInTheDocument();
    expect(screen.getByRole("button", { name: "Да" })).toHaveFocus();
    await userEvent.click(screen.getByRole("button", { name: "Да" }));
    expect(onConfirm).toHaveBeenCalledOnce();
  });

  it("goes back on No and on Escape", async () => {
    const onConfirm = vi.fn();
    render(<ConfirmButton onConfirm={onConfirm}>Откажи</ConfirmButton>);
    await userEvent.click(screen.getByRole("button", { name: "Откажи" }));
    await userEvent.click(screen.getByRole("button", { name: "Не" }));
    expect(screen.getByRole("button", { name: "Откажи" })).toBeInTheDocument();
    await userEvent.click(screen.getByRole("button", { name: "Откажи" }));
    await userEvent.keyboard("{Escape}");
    expect(screen.getByRole("button", { name: "Откажи" })).toBeInTheDocument();
    expect(onConfirm).not.toHaveBeenCalled();
  });

  it("never calls window.confirm", async () => {
    const spy = vi.spyOn(window, "confirm");
    render(<ConfirmButton onConfirm={() => {}}>Откажи</ConfirmButton>);
    await userEvent.click(screen.getByRole("button", { name: "Откажи" }));
    await userEvent.click(screen.getByRole("button", { name: "Да" }));
    expect(spy).not.toHaveBeenCalled();
  });
});
