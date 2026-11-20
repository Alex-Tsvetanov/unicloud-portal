# Proposal: sharing the portal's UI with the other UniCloud modules

Status: proposal for Teams 2 to 5 (2026-11-20, updated 2026-11-21). From Team 1, "Portal and Identity". Variants A and C are now built and served from the portal ([ADR 0004](../adr/0004-react-ui-with-a-json-api.md)).

## Why

Students move between the portal and your modules all day. If every module looks different and shows the same data in a different way, UniCloud feels like five products, not one. Team 1 can share its look and some of its components. How best to do it depends on how your module renders its pages, which we do not know yet. So here are four variants. Pick one or more, and answer the questions at the end.

## What the portal already is

The portal is a Rust server with a React front end at `/app`. Its pages are built from React components that we publish as a package, `@unicloud/ui`, so your team can use the same ones. Single sign-on and the directory API are already live on staging: https://unicloud-portal-staging-tszc.onrender.com/api-docs

Two facts shape every variant:

- **Your modules are on other sites.** A browser does not send the portal's session cookie to requests your page makes to the portal. So your page cannot ask the portal for "the signed-in user's" HTML directly from the browser.
- **The portal can be asleep or busy.** Staging sleeps after 15 minutes. At 08:00 on enrollment day everyone is loading pages. A variant that makes your page wait for the portal ties your uptime to ours.

## The variants

### Variant A: shared look (CSS only)

The portal publishes one versioned stylesheet: colours, fonts, spacing, light and dark mode, and classes for the common pieces (top bar, cards, tables, status badges, the seats meter, buttons, forms).

```html
<link rel="stylesheet" href="https://<portal>/ui/v1/unicloud.css">

<body class="uc-root">
  <span class="uc-badge uc-badge-ok">Действащ</span>
  <button class="uc-btn uc-btn-primary">Запиши се</button>
</body>
```

- **Works with:** anything that outputs HTML. Thymeleaf, JSP, React, Angular, Vue.
- **You build:** your own markup, using our class names.
- **Runtime link to the portal:** none after the first load (the file is cached; a new major version gets a new path, `v2`). You can also copy the file into your own static files.
- **Status:** built. Live at `/ui/v1/unicloud.css`. The class names are listed in the package README.
- **Effort:** Team 1 done; your team small.

### Variant B: server-fetched HTML fragments

For data the portal owns (a student card, a student's enrollments, a course's roster), the portal serves ready HTML. Your backend fetches it with your directory token and puts it into your page. The browser never talks to the portal.

```java
// Spring: fetch once, cache for a minute, put it into the page
String card = restClient.get()
    .uri(portal + "/ui/fragments/v1/student-card/{sub}", sub)
    .header("Authorization", "Bearer " + directoryToken())
    .retrieve().body(String.class);
model.addAttribute("studentCard", card);
```

```html
<!-- Thymeleaf -->
<div th:utext="${studentCard}"></div>
```

- **Works with:** server-rendered pages (Thymeleaf, JSP, Freemarker). Possible in React with `dangerouslySetInnerHTML`, but awkward.
- **You build:** one HTTP call and a cache.
- **Runtime link to the portal:** one server-to-server call per fragment, which you can cache. If the portal is down, leave the piece out.
- **Buttons inside a fragment** (for example "enroll") are links to the portal's own page. The user lands there signed in, through single sign-on. A button that acted directly from your site would arrive without the portal's session, so the portal would refuse it.
- **Effort:** Team 1 medium; your team small.

### Variant C: React components (`@unicloud/ui`)

The portal publishes its own React components as an npm package. They only draw; they get their data from you (from the ID token claims or from the directory API your backend already calls). No cookies, no tokens in the browser. No npm account or registry is needed: the portal serves the package file.

```sh
npm install https://<portal>/ui/v1/unicloud-ui-1.0.0.tgz
```

```tsx
import "@unicloud/ui/unicloud.css";
import { TopBar, StatusBadge, SeatsMeter, CourseCard } from "@unicloud/ui";

<StatusBadge status={student.status} />
<SeatsMeter taken={18} capacity={30} />
<CourseCard code="MATH1" title="Линейна алгебра" seats={{ taken: 18, capacity: 30 }} />
```

Components in 1.0.0: `TopBar`, `PageHeader`, `Card`, `Button`, form fields, `Alert`, `StatusBadge`, `RoleBadge`, `SeatsMeter`, `DataTable`, `EmptyState`, `Stat`, `CourseCard`, `StudentCard`, `ConfirmButton`. The README in the package shows each one.

- **Works with:** React 19 apps. Angular and Vue apps can use variant A's classes instead.
- **You build:** you pass the data in, and keep your own routing and state.
- **Runtime link to the portal:** none. The package is part of your build.
- **Status:** built. The portal itself uses these components.
- **Effort:** Team 1 done; your team small.

### Variant D: data only

You use the directory API and the claims as JSON and build your own UI. This already works today: https://unicloud-portal-staging-tszc.onrender.com/api-docs

- **Works with:** everything.
- **You build:** all of the UI.
- **Runtime link to the portal:** the API calls you already make.
- **Effort:** Team 1 none; your team the most. Modules will look different.

## Side by side

| | A: CSS | B: fragments | C: React package | D: data only |
|---|---|---|---|---|
| Server-rendered Java (Thymeleaf, JSP) | yes | best fit | yes | yes |
| React | yes | awkward | best fit | yes |
| Angular, Vue | yes | awkward | no (use A) | yes |
| Same look as the portal | close | exact | exact | no |
| Your page waits for the portal | no | yes, from your server, cacheable | no | only for data |
| Portal data rendered for you | no | yes | no (you pass it) | no |
| Tokens or cookies in the browser | no | no | no | no |
| Work for Team 1 | done | medium | done | none |
| Work for your team | small | small | small | large |

## What Team 1 recommends

- **Everyone:** A. It is the cheapest way to look like one product, and it costs nothing at run time.
- **Server-rendered modules:** add B for the few pieces where the portal owns the data (student card, enrollments, roster).
- **React apps:** add C.
- D stays available to everyone in any case.

A and C are built. We build B only if at least one team will use it. Nothing here changes single sign-on or the directory API.

## What we need from you

Please answer in the team channel or as a comment:

1. How does your module render its pages? (Thymeleaf or JSP, React, Angular, Vue, other)
2. Which variants would you use? (A, B, C, D) If C: which React version do you use?
3. Which pieces do you need? (top bar, status badge, seats meter, course card, student card, enrollment list, roster, other)
4. With B: can your backend cache a fragment for a minute, and leave it out when the portal does not answer?
5. Your module's address on staging, so we can test with you.
