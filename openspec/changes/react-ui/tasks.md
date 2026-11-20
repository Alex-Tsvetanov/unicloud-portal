# Tasks: React UI

## 1. Contract
- [x] 1.1 ADR 0004
- [x] 1.2 `/api/v1/portal/*` in `api/openapi.json` ("Portal UI" tag, `portalSession` and `csrfHeader` schemes)

## 2. Server
- [x] 2.1 `portal_api`: session, options, profiles, catalogue, enrollments, teaching, faculty, inspection, staff, modules
- [x] 2.2 JSON errors, JSON CSRF failures, `no-store` on module registration
- [x] 2.3 Serve the app at `/app` and the UI package at `/ui/v1`
- [x] 2.4 Sign-in `next` return path, `/` sends signed-in users to `/app/`
- [x] 2.5 Remove the Maud page modules; keep sign-in, password, logout confirmation, errors, staging

## 3. Frontend
- [x] 3.1 `@unicloud/ui` 1.0.0 with tests and a README for other teams
- [x] 3.2 The portal app: every page the Maud UI had, with tests
- [x] 3.3 Types generated from `api/openapi.json`

## 4. Build and checks
- [x] 4.1 `tests/portal_api.rs`: every role through the API, admin, return path, hosting
- [x] 4.2 Old page tests moved to the API
- [x] 4.3 Docker Node stage, CI `web` job
- [x] 4.4 Checked in a browser on a local server: sign-in return, catalogue, enroll, cancel, sign-out, every staff and inspector form, module secret, status change, teacher reject, phone width
- [x] 4.4a Installed `@unicloud/ui` from the portal in an empty project (TypeScript build and import pass; React stays external)
- [x] 4.5 Checked on staging after deploy (4736568): app, 401 JSON, package and stylesheet with CORS, missing asset 404, package installs from staging
- [ ] 4.6 Ask the other teams which components they need (see `docs/proposals/ui-sharing.md`)
