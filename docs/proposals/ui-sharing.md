# Proposal: sharing the portal's UI with the other UniCloud modules

Status: proposal for Teams 2 to 5 (2026-11-20). From Team 1, "Portal and Identity".

## Why

Students move between the portal and your modules all day. If every module looks different and shows the same data in a different way, UniCloud feels like five products, not one. Team 1 can share its look and some of its components. How best to do it depends on how your module renders its pages, which we do not know yet. So here are four variants. Pick one or more, and answer the questions at the end.

## What the portal already is

The portal is a Rust server. Its pages are built from small server-side components (like React components, but they run on the server and send finished HTML). htmx swaps parts of a page in place, so the page feels fast with almost no JavaScript. Single sign-on and the directory API are already live on staging: https://unicloud-portal-staging-tszc.onrender.com/api-docs

Two facts shape every variant:

- **Your modules are on other sites.** A browser does not send the portal's session cookie to requests your page makes to the portal. So your page cannot ask the portal for "the signed-in user's" HTML directly from the browser.
- **The portal can be asleep or busy.** Staging sleeps after 15 minutes. At 08:00 on enrollment day everyone is loading pages. A variant that makes your page wait for the portal ties your uptime to ours.

## The variants

### Variant A: shared look (CSS only)

The portal publishes one versioned stylesheet: colours, fonts, spacing, light and dark mode, and classes for the common pieces (top bar, cards, tables, status badges, the seats meter, buttons, forms).

```html
<link rel="stylesheet" href="https://<portal>/ui/v1/unicloud.css">

<span class="uc-badge uc-badge-ok">Действащ</span>
<div class="uc-seats" style="--taken: 18; --capacity: 30">12 / 30</div>
```

- **Works with:** anything that outputs HTML. Thymeleaf, JSP, React, Angular, Vue.
- **You build:** your own markup, using our class names.
- **Runtime link to the portal:** none after the first load (the file is cached for a year; a new version gets a new path, `v2`).
- **Effort:** Team 1 small; your team small.

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

### Variant C: web components

The portal publishes a small script with standard custom elements. They only draw; they get their data from you (from the ID token claims or from the directory API your backend already calls). No cookies, no tokens in the browser.

```html
<script type="module" src="https://<portal>/ui/v1/unicloud.js"></script>

<unicloud-header name="Александър Стоянов" role="STUDENT" portal="https://<portal>"></unicloud-header>
<unicloud-status status="INTERRUPTED"></unicloud-status>
<unicloud-seats taken="18" capacity="30"></unicloud-seats>
<unicloud-course code="MATH1" title="Линейна алгебра" semester="2026-27-W"></unicloud-course>
```

```jsx
// React 19 uses custom elements directly
<unicloud-status status={student.status} />
```

- **Works with:** everything, including React, Angular and Vue apps and plain Thymeleaf pages.
- **You build:** you pass the data in.
- **Runtime link to the portal:** none after the first load (cached, versioned).
- **Effort:** Team 1 medium; your team small.

### Variant D: data only

You use the directory API and the claims as JSON and build your own UI. This already works today: https://unicloud-portal-staging-tszc.onrender.com/api-docs

- **Works with:** everything.
- **You build:** all of the UI.
- **Runtime link to the portal:** the API calls you already make.
- **Effort:** Team 1 none; your team the most. Modules will look different.

## Side by side

| | A: CSS | B: fragments | C: web components | D: data only |
|---|---|---|---|---|
| Server-rendered Java (Thymeleaf, JSP) | yes | best fit | yes | yes |
| React, Angular, Vue | yes | awkward | best fit | yes |
| Same look as the portal | close | exact | exact | no |
| Your page waits for the portal | no | yes, from your server, cacheable | no | only for data |
| Portal data rendered for you | no | yes | no (you pass it) | no |
| Tokens or cookies in the browser | no | no | no | no |
| Work for Team 1 | small | medium | medium | none |
| Work for your team | small | small | small | large |

## What Team 1 recommends

- **Everyone:** A. It is the cheapest way to look like one product, and it costs nothing at run time.
- **Server-rendered modules:** add B for the few pieces where the portal owns the data (student card, enrollments, roster).
- **Single-page apps:** add C.
- D stays available to everyone in any case.

We build only what at least one team will use. Nothing here changes single sign-on or the directory API.

## What we need from you

Please answer in the team channel or as a comment:

1. How does your module render its pages? (Thymeleaf or JSP, React, Angular, Vue, other)
2. Which variants would you use? (A, B, C, D)
3. Which pieces do you need? (top bar, status badge, seats meter, course card, student card, enrollment list, roster, other)
4. With B: can your backend cache a fragment for a minute, and leave it out when the portal does not answer?
5. Your module's address on staging, so we can test with you.
