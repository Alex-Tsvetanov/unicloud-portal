# @unicloud/ui

Shared React components and design tokens for UniCloud modules. Team 1 (Portal and Identity) builds
and serves this package so every team's pages look and behave the same.

The components are presentational only. They do not fetch data and do not depend on a router. Your
app owns data, routing and state. Every CSS class starts with `uc-`, so the styles do not clash with
your own.

## Install

The portal serves the package as an npm tarball. Replace `<portal>` with the portal's address.

```sh
npm install https://<portal>/ui/v1/unicloud-ui-1.0.0.tgz
```

The package needs React 19 (`react` and `react-dom` are peer dependencies).

Import the stylesheet once, in your entry file:

```tsx
import "@unicloud/ui/unicloud.css";
```

Add the class `uc-root` to `<body>` (or to your top wrapper) to get the base font, colours and focus
rings. Wrap page content in `<main className="uc-main">` for the standard width and padding.

### Without React

The same stylesheet is served as plain CSS at `https://<portal>/ui/v1/unicloud.css`. Server-rendered
apps (for example Spring with Thymeleaf) can link it and use the `uc-` classes directly. The class
names for each component are listed below.

## Components

All examples assume:

```tsx
import { Button, Card /* ... */ } from "@unicloud/ui";
```

### TopBar

The top navigation. Pass your nav items. Pass `renderLink` to use your router's link component.

```tsx
<TopBar
  brand="UniCloud"
  items={[
    { label: "Начало", href: "/home", active: true },
    { label: "Курсове", href: "/courses" },
  ]}
  right={<span>Мария Петрова</span>}
  renderLink={({ item, className, children }) => (
    <Link to={item.href} className={className} aria-current={item.active ? "page" : undefined}>
      {children}
    </Link>
  )}
/>
```

Classes: `uc-topbar`, `uc-brand`, `uc-brand-mark`, `uc-nav`, `uc-who`.

### PageHeader

```tsx
<PageHeader kicker="Студент" title="Курсове" lede="Изберете курс." actions={<Button>Ново</Button>} />
```

Classes: `uc-page-header`, `uc-kicker`, `uc-h1`, `uc-lede`.

### Card

```tsx
<Card title="Профил" aside={<RoleBadge role="STUDENT" />}>
  Съдържание
</Card>
```

Classes: `uc-card`, `uc-card-head`, `uc-h2`.

### Button

Variants: `primary` (default), `secondary`, `link`, `danger`. Sizes: `sm`, `md` (default), `lg`.
`busy` disables the button while a request runs. Buttons default to `type="button"`.

```tsx
<Button onClick={save}>Запази</Button>
<Button variant="secondary" size="sm">Назад</Button>
<Button variant="danger" busy={isSaving}>Изтрий</Button>
```

To make a link look like a button, use `buttonClass`:

```tsx
<a href="/courses" className={buttonClass("secondary", "sm")}>Курсове</a>
```

Classes: `uc-btn`, `uc-btn-primary`, `uc-btn-secondary`, `uc-btn-link`, `uc-btn-danger`, `uc-btn-sm`, `uc-btn-lg`.

### Field, TextInput, Select, TextArea, Checkbox

`Field` adds the label, a hint and an error message, and links them to the control for screen
readers.

```tsx
<Field label="Имейл" hint="Университетски адрес" error={emailError}>
  <TextInput name="email" type="email" required />
</Field>

<Field label="Факултет">
  <Select name="faculty" options={[{ value: "FKSU", label: "ФКСУ" }]} />
</Field>

<Field label="Бележка">
  <TextArea name="note" />
</Field>

<Checkbox name="directory" label="Достъп до директорията" />
```

Classes: `uc-field`, `uc-label`, `uc-hint`, `uc-field-error`, `uc-input`, `uc-select`, `uc-textarea`, `uc-check`.

### Alert and Flash

Tones: `ok`, `error`, `info`. Errors use `role="alert"`, the others `role="status"`.

```tsx
<Alert tone="error">Нещо се обърка.</Alert>
<Alert tone="ok" onClose={() => setOpen(false)}>Запазено.</Alert>
```

`Flash` shows one result message, or nothing when the message is `null`:

```tsx
const [flash, setFlash] = useState<FlashMessage | null>(null);
<Flash message={flash} onClose={() => setFlash(null)} />
```

Classes: `uc-alert`, `uc-alert-ok`, `uc-alert-error`, `uc-alert-info`.

### StatusBadge and RoleBadge

Known values get a colour and a Bulgarian label. **Unknown values are rendered as-is, in a neutral
colour**, so your module can pass its own statuses and roles. Pass `label` to set the text yourself.

```tsx
<StatusBadge status="enrolled" />          {/* Записан, green */}
<StatusBadge status="archived" />          {/* archived, neutral */}
<RoleBadge role="teacher" />               {/* Преподавател */}
<RoleBadge role="librarian" label="Библиотекар" />
```

Known statuses: `enrolled`, `active`, `approved`, `open`, `pending`, `waitlisted`, `cancelled`,
`canceled`, `closed`, `disabled`, `rejected`, `full`. Known roles: `student`, `teacher`, `dean`,
`admin`, `faculty_admin`, `inspector`. Matching ignores case.

Classes: `uc-badge` plus one of `uc-badge-ok`, `uc-badge-warn`, `uc-badge-err`, `uc-badge-info`,
`uc-badge-neutral`, `uc-badge-accent`.

### SeatsMeter

```tsx
<SeatsMeter taken={27} capacity={30} />
```

It turns amber at 80% and red when full. Classes: `uc-seats`, `uc-seats-track`, `uc-seats-fill`,
with `data-level="ok|warn|full"`.

### DataTable

```tsx
<DataTable
  columns={[
    { key: "name", header: "Име", render: (r) => r.name },
    { key: "year", header: "Курс", numeric: true, render: (r) => r.year },
  ]}
  rows={students}
  rowKey={(r) => r.id}
  empty={<EmptyState title="Няма студенти" />}
/>
```

Classes: `uc-table-wrap`, `uc-table`, `uc-num`.

### EmptyState

```tsx
<EmptyState title="Няма записвания" action={<Button>Разгледай курсовете</Button>}>
  Още не сте записани за курс.
</EmptyState>
```

Classes: `uc-empty`, `uc-empty-title`.

### Stat

```tsx
<Stat label="Студенти" value={412} hint="този семестър" />
```

Classes: `uc-stat`, `uc-stat-label`, `uc-stat-value`, `uc-stat-hint`.

### CourseCard

```tsx
<CourseCard
  code="CS101"
  title="Програмиране"
  status="open"
  meta={["ФКСУ", "доц. Иванов", "5 ECTS"]}
  seats={{ taken: 3, capacity: 30 }}
  actions={<Button size="sm">Запиши се</Button>}
/>
```

Classes: `uc-course`, `uc-course-code`, `uc-course-title`, `uc-course-meta`, `uc-card-foot`.

### StudentCard

```tsx
<StudentCard name="Мария Петрова" meta={["121220001", "КСТ, 2 курс"]} badges={<StatusBadge status="active" />} />
```

Classes: `uc-student`, `uc-avatar`, `uc-student-name`, `uc-student-meta`.

### ConfirmButton

A two-step confirm inside the page. The first click asks the question. The second click confirms.
Escape or "Не" goes back. It never opens a browser `confirm()` dialog.

```tsx
<ConfirmButton question="Да откажа ли записването?" onConfirm={() => cancel(id)}>
  Откажи
</ConfirmButton>
```

Classes: `uc-confirm`, `uc-confirm-q`.

## Theming

The stylesheet follows the system light or dark setting. To force a theme, set `data-theme` on
`<html>` (or any wrapper):

```html
<html data-theme="dark">
```

All colours are CSS variables, so you can change them after the import:

```css
:root {
  --uc-accent: #1f4e8c;
  --uc-accent-ink: #fff;
}
```

Main tokens: `--uc-paper`, `--uc-card`, `--uc-ink`, `--uc-ink-2`, `--uc-rule`, `--uc-accent`,
`--uc-accent-ink`, `--uc-ok`, `--uc-warn`, `--uc-err`, `--uc-info` (each status colour also has a
`-bg` variant), `--uc-focus`, `--uc-radius-s`, `--uc-radius-l`, `--uc-serif`, `--uc-sans`, `--uc-mono`.

Focus rings are always visible for keyboard users. Animations turn off when the user asks for reduced
motion.

## Building this package

From `web/`:

```sh
npm install
npm run test -w @unicloud/ui
npm run pack:ui     # writes web/dist-ui/unicloud-ui-1.0.0.tgz
```
