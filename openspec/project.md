# Project context

## Purpose
UniCloud Portal is Team 1's product in the UniCloud course project at TU-Sofia (MCSE06). It is the university's single entry point: accounts, academic profiles, course offerings and enrollment. It is also the OpenID Connect provider that every other UniCloud module signs its users in through, and the directory those modules read who is who from.

Vocabulary: [GLOSSARY.md](../GLOSSARY.md). Decisions: [docs/adr](../docs/adr/). Product requirements: [docs/prd.md](../docs/prd.md).

## Tech stack
- Rust (stable), axum 0.8, tower-http, tower-sessions with a PostgreSQL store
- sqlx 0.9 on PostgreSQL 17, migrations embedded and applied at start, offline query data committed
- jsonwebtoken 12 (pure Rust backend) for RS256, argon2 for passwords
- Maud components and htmx 2.0.x, served from the binary, no JavaScript build
- Tests: cargo test, axum-test, reqwest, testcontainers
- Docker (cargo-chef, slim glibc base). Staging on Render (Blueprint `render.yaml`). Production on Azure Container Apps

## Conventions
- UI text is Bulgarian. Code, API, logs and docs are English. Wire names use US spelling ("enrollment").
- Published names in `docs/sso.md` never change. The contract only grows.
- Every state-changing form has CSRF protection. Every handler checks a permission, not a role.
- Docs follow the writing rules: short sentences, plain words, no em or en dashes.
- Conventional commits. One branch per change, merged to `main` by pull request.

## Domain context
Roles: Student, Teacher, Dean, Inspector, Admin. A faculty has specialties, teachers, one dean and inspectors. A course offering is one course in one semester with one teacher and limited places. A student's enrollment request takes a free place at once.

## External dependencies
- Module teams 2 to 5 code against [docs/sso.md](../docs/sso.md).
- Render free web service and PostgreSQL (staging), Azure Container Apps and PostgreSQL Flexible Server (production).
