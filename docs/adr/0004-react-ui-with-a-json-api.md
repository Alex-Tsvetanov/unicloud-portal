# React UI with a JSON API; reusable components for every team

Status: accepted (2026-11-21). Supersedes the UI part of [ADR 0001](0001-rust-rewrite-with-hypermedia-ui.md) (Maud components with htmx). The Rust server, the OIDC provider and everything else in ADR 0001 stay.

The team chose React for the portal's UI so that the same components can be reused by the other UniCloud modules that build with React, in this iteration. A React app and its components can travel to another team's codebase; server-rendered Maud components cannot. This drops the original brief's "hypermedia" wording on purpose: the browser now draws pages from JSON.

## The shape

- `web/packages/ui` is `@unicloud/ui`: presentational React components and the design tokens (CSS). Components take data as props and never call the portal, so another module can use them with its own data. It builds as an ES module with TypeScript types and one stylesheet.
- `web/apps/portal` is the portal's single-page app (React 19, TypeScript, Vite, React Router, TanStack Query). It is served by the Rust binary under `/app`.
- The app talks to `/api/v1/portal/*`, a JSON API on the same site. It uses the same session cookie (ADR 0003) and sends the CSRF token in the `X-CSRF-Token` header on every change. Every permission check stays on the server.
- The TypeScript types for the API are generated from `api/openapi.json`, so the frontend and the server cannot drift apart unnoticed.
- The sign-in, password-change and logout-confirmation pages, the staging page and `/api-docs` stay server-rendered. A module's login never waits for a JavaScript bundle.

## Distribution to other teams

The portal serves the packed library itself at `/ui/v1/unicloud-ui-<version>.tgz`. Teams download the file into their repository and run `npm install ./vendor/unicloud-ui-<version>.tgz`, so their builds never depend on the portal being awake (npm 12 also refuses URL installs unless `--allow-remote=all` is set). No npm registry or account is needed, and a version never changes once published. The same stylesheet is served at `/ui/v1/unicloud.css` for teams that do not use React (variant A of the UI-sharing proposal).

## Consequences

- There is now a JavaScript build: Node runs in a Docker build stage and in CI. The runtime is still one Rust binary.
- The first visit downloads the app bundle (React plus the app). It is cached afterwards; keep it small.
- The Maud page modules are removed. `src/ui/components.rs` stays for the few server-rendered pages.
