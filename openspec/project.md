# Project context

## Purpose
UniCloud Portal is Team 1's product in the UniCloud course project at TU-Sofia (MCSE06). It is the university's single entry point: accounts, academic profiles, course offerings and enrollment. It is also the OpenID Connect provider that every other UniCloud module signs its users in through, and the directory those modules read who is who from.

Vocabulary: [GLOSSARY.md](../GLOSSARY.md). Decisions: [docs/adr](../docs/adr/). Product requirements: [docs/prd.md](../docs/prd.md).

## Tech stack
- Rust (stable), axum 0.8, tower-http, tower-sessions with a PostgreSQL store
- sqlx 0.9 on PostgreSQL 17, migrations embedded and applied at start under an advisory lock; queries checked at run time by the tests
- jsonwebtoken 12 (pure Rust backend) for RS256, argon2 for passwords, SHA-256 for client secrets
- React 19, TypeScript, Vite, React Router and TanStack Query for the UI (`web/apps/portal`, served at `/app`); `@unicloud/ui` for the shared components (`web/packages/ui`, served at `/ui/v1/`). Maud only for sign-in, password, logout confirmation, errors and the staging page
- Tests: cargo test; Vitest with Testing Library for the frontend; black-box suites start the real router on a random port and drive it with reqwest against PostgreSQL
- Docker (cargo-chef, slim glibc base). Staging on Render (Blueprint `render.yaml`). Production on Azure Container Apps

## Conventions
- UI text is Bulgarian. Code, API, logs and docs are English. Wire names use US spelling ("enrollment").
- Published names in `docs/sso.md` never change. The contract only grows.
- Every state-changing request has CSRF protection (`X-CSRF-Token` from the app, a form field on server pages). Every handler checks a permission, not a role.
- Docs follow the writing rules: short sentences, plain words, no em or en dashes.
- Conventional commits. One branch per change, merged to `main` by pull request.

## Domain context
Roles: Student, Teacher, Dean, Inspector, Admin. A faculty has specialties, teachers, one dean and inspectors. A course offering is one course in one semester with one teacher and limited places. A student's enrollment request takes a free place at once.

## External dependencies
- Module teams 2 to 5 code against [docs/sso.md](../docs/sso.md).
- Render free web service and PostgreSQL (staging), Azure Container Apps and PostgreSQL Flexible Server (production).
