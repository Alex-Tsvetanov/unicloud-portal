# Change: React UI with a JSON API, and shared components for other teams

## Why
Most UniCloud teams build their modules in Java, and some build React front ends. The portal's Maud components could not travel to their codebases. The team chose to rebuild the portal's UI in React, so the same components can be packaged and reused by other teams in this iteration ([ADR 0004](../../../docs/adr/0004-react-ui-with-a-json-api.md)). The options considered are in [docs/proposals/ui-sharing.md](../../../docs/proposals/ui-sharing.md).

## What changes
- **BREAKING (UI paths):** the portal's pages move under `/app` (for example `/courses` becomes `/app/courses`). `/` sends a signed-in user to `/app/`. Modules do not link to these pages, so no module changes.
- New JSON API at `/api/v1/portal/*` for the app, on the same site, with the same session cookie and the CSRF token in the `X-CSRF-Token` header. Documented in `api/openapi.json` under the "Portal UI" tag.
- New React app (`web/apps/portal`) served at `/app`. TypeScript types are generated from `api/openapi.json`.
- New package `@unicloud/ui` 1.0.0 (`web/packages/ui`): presentational React components and the `uc-` stylesheet. The portal serves the package at `/ui/v1/unicloud-ui-1.0.0.tgz` and the stylesheet at `/ui/v1/unicloud.css`.
- Sign-in accepts `next=/app/...` and returns there after sign-in. Nothing outside `/app/` is accepted.
- Removed: the Maud page modules and htmx. Kept server-rendered: sign-in, password change, logout confirmation, error pages, `/staging`, `/api-docs`.
- Docker gains a Node build stage; CI gains a `web` job.

## Impact
- Specs: new `portal-ui`. `role-access` and `course-catalog` scenarios that said "page" now hold for the matching API call (same permission, same Bulgarian message).
- Code: `src/portal_api/`, `src/spa.rs`, `src/web/csrf.rs`, `src/accounts/handlers.rs`, `src/ui/` (reduced), `web/`.
- Docs: ADR 0004, README, design, `docs/proposals/ui-sharing.md`, `web/packages/ui/README.md`.
