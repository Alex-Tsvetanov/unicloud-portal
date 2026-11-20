# UniCloud Portal

The Unified University Portal: accounts, academic profiles, the course catalogue and enrollment. It is also the single sign-on provider for every other UniCloud module, and the directory those modules read who is who from.

**Stack:** Rust · axum · sqlx · PostgreSQL 17 · Maud components with htmx (server-rendered, no JavaScript build) · Docker · GitHub Actions · Render (staging) · Azure Container Apps (production)

**Staging (Render):** https://unicloud-portal-staging-tszc.onrender.com (staging tools at `/staging`)

**Design and decisions:** [docs/prd.md](docs/prd.md) (what and why), [openspec/changes/rust-rewrite/design.md](openspec/changes/rust-rewrite/design.md) (how), [openspec/changes/rust-rewrite/specs/](openspec/changes/rust-rewrite/specs/) (the requirements, testable), [docs/adr/](docs/adr/) (decisions), [GLOSSARY.md](GLOSSARY.md) (words), [docs/slo.md](docs/slo.md) (the uptime promise), [docs/sso.md](docs/sso.md) (for module teams).

## Contents

1. [How it works](#how-it-works)
2. [What's in the repo](#whats-in-the-repo)
3. [Run it locally](#run-it-locally)
4. [Everyday workflow](#everyday-workflow)
5. [The pipeline](#the-pipeline)
6. [Settings](#settings)
7. [Changing the database](#changing-the-database)
8. [Rules the code must follow](#rules-the-code-must-follow)
9. [Load testing](#load-testing)
10. [Troubleshooting](#troubleshooting)

## How it works

```mermaid
flowchart LR
    B[Browser] -->|HTML + htmx, one session cookie| P[Portal replicas]
    M[Module backends] -->|OIDC: /oauth2/token, /userinfo| P
    M -->|/api/v1/directory, client_credentials| P
    P --> DB[(PostgreSQL)]
```

- One stateless Rust binary. Any replica answers any request. Sessions, codes and logins live in PostgreSQL.
- Pages are rendered on the server. Each page is built from Maud components (Rust functions that return HTML). htmx swaps fragments in place: the catalogue search, enrolling, cancelling and rejecting work without a full page load, and every page still works as plain forms.
- One cookie, `__Host-UNICLOUD_SSO`, serves the portal UI and single sign-on ([ADR 0003](docs/adr/0003-one-cookie-session.md)).
- The OpenID Connect provider is our own code ([ADR 0001](docs/adr/0001-rust-rewrite-with-hypermedia-ui.md)). Its behaviour is pinned by `tests/sso_conformance.rs` and [docs/sso-wire-contract.md](docs/sso-wire-contract.md).
- Enrollment takes a place in one conditional `UPDATE`, so the 08:00 rush cannot oversell (`tests/portal.rs` fires 40 students at 10 places).

## What's in the repo

```text
src/
  main.rs, lib.rs       start-up, routes, graceful shutdown
  config.rs             every setting, from environment variables
  access.rs             roles and permissions (bit positions are part of the contract)
  accounts/             users, passwords (Argon2id), sign-in, staff and student creation
  enrollment.rs         enroll, cancel, reject without overselling
  oidc/                 the SSO provider: discovery, keys, authorize, token, userinfo, logout, modules
  directory.rs          /api/v1/directory for modules
  staging.rs            /staging and /api/v1/staging (staging and development only)
  ui/                   Maud components and pages
  web/                  session, current user, CSRF
  ops.rs                /health, X-Served-By, hourly cleanup
migrations/             SQL migrations, applied at start
assets/                 htmx.min.js (2.0.11), app.css, app.js
config/                 sso-callback-hosts.txt: production's allowed callback hosts
tests/                  black-box suites, the Java staging recording, a test-only key
docs/, openspec/        PRD, design, specs, ADRs, SLO, SSO guide
render.yaml             the staging Blueprint
infra/setup-azure.sh    one-time Azure setup (DevOps)
loadtest/spike.js       k6 test of the 08:00 rush
```

## Run it locally

You need Rust (stable), Docker and Git.

1. Start PostgreSQL 17:

   ```bash
   docker run --name unicloud-rust-pg -d -p 55432:5432 -e POSTGRES_PASSWORD=dev -e POSTGRES_DB=unicloud postgres:17
   ```

2. Start the portal with demo data (`scripts/dev.ps1` on Windows):

   ```bash
   scripts/dev.sh
   ```

3. Open http://localhost:8090. The demo accounts all use the password `demo-pass-123`:

   | Role | Email |
   |---|---|
   | Student | `student1@student.tu-sofia.bg` to `student5@...` |
   | Teacher | `ivan.petrov@tu-sofia.bg` |
   | Dean (FCST) | `dean.fcst@tu-sofia.bg` |
   | Inspector | `inspector.fcst@tu-sofia.bg` |
   | Admin | `admin@tu-sofia.bg` |

   The staging tools are at http://localhost:8090/staging.

4. Run the tests (they use the same database):

   ```bash
   cargo test
   ```

The scripts use `tests/fixtures/test-only-signing-key.pem`. It exists only for tests and local runs. Never use it on staging or production.

## Everyday workflow

`main` is protected. Every change goes through a pull request whose `test` and `build` checks must pass.

1. Branch from the latest `main`: `git switch main && git pull && git switch -c feature/short-description`.
2. Make the change. Run `cargo fmt`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.
3. If the change touches a requirement, update its spec in `openspec/`. If it changes what modules see, update `docs/sso.md`.
4. Push, open a pull request into `main`, wait for green checks and a teammate's approval, then squash and merge.
5. Render deploys `main` to staging a few minutes later. Try the change there.
6. Production (team repository only): a reviewer of the `production` environment approves the waiting `deploy` job.

## The pipeline

Defined in `.github/workflows/deploy.yml`.

| Job | Runs on | What it does |
|---|---|---|
| `test` | Every pull request and every merge | `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test` against PostgreSQL 17 |
| `build` | After `test` | Builds the Docker image. On `main` it pushes it to GHCR, tagged with the commit |
| `deploy-staging` | Merges to `main` | Calls Render's deploy hook if `RENDER_DEPLOY_HOOK` is set. Otherwise the Blueprint deploys `main` by itself |
| `deploy` | Merges to `main` in `rata5/unicloud-portal`, after approval | Deploys the image to Azure Container Apps (2 to 10 replicas) and waits until `/health` shows the commit |

Production secrets (team repository, **Settings → Secrets and variables → Actions**): `AZURE_CLIENT_ID`, `AZURE_TENANT_ID`, `AZURE_SUBSCRIPTION_ID`, `GHCR_PAT`, `DATABASE_URL` (a `postgres://` URL from `infra/setup-azure.sh`) and `JWT_PRIVATE_KEY` (the RSA private key in PKCS#8 PEM that signs tokens).

## Settings

| Variable | Meaning | Default |
|---|---|---|
| `APP_ENV` | `production`, `staging` or `development`. Not `production` turns on the staging tools and the `any` callback rule | `production` |
| `DATABASE_URL` | `postgres://user:password@host:port/db` | required |
| `APP_DB_SCHEMA` | PostgreSQL schema for the tables | `public` |
| `APP_DB_POOL_MAX` | Connections per replica | `10` |
| `APP_SIGNING_KEY_PEM` | RSA private key (PKCS#8 PEM). The public key and `kid` are derived from it | required |
| `APP_SSO_ISSUER` | The public `https` address, used as `iss`. Falls back to `RENDER_EXTERNAL_URL`, then to the request | |
| `APP_SSO_ALLOWED_CALLBACK_HOSTS` | Replaces `config/sso-callback-hosts.txt` (production) | |
| `APP_CURRENT_SEMESTER` | The semester the catalogue shows | `2026-27-W` |
| `APP_DEMO_PASSWORD` | Staging and development: seed demo people and courses with this password | |
| `APP_BOOTSTRAP_ADMIN_EMAIL`, `APP_BOOTSTRAP_ADMIN_PASSWORD` | Create the first admin if none exists. The password must be changed at first sign-in | |
| `PORT` | Listen port | `8080` |

To make a signing key:

```bash
openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -out signing-key.pem
```

Paste the whole file into the secret, then delete the file. Changing the key logs every module user out, so do it between semesters.

## Changing the database

Add a new file in `migrations/` with the next number, for example `0003_add_x.sql`. Never edit a migration that has run anywhere. The portal applies new migrations at start, under a database lock, before it takes traffic.

Old and new replicas run side by side for a minute during every deploy, so each change must work with the previous code. To rename or remove a column, add the new one in one release and drop the old one in a later release.

## Rules the code must follow

- **Keep `/health` cheap.** It never touches the database, and the deploy waits until it shows the new commit. Every response carries `X-Served-By` with the replica's name.
- **Never change a published name.** Roles, permission names and their bit positions, claim names and endpoint paths in `docs/sso.md` only grow ([ADR 0002](docs/adr/0002-grow-the-sso-contract.md)).
- **Check permissions, not roles.** Every handler asks `user.can(Permission::...)`.
- **Every state-changing form carries the CSRF token.** The layout sends it automatically for htmx requests.
- **UI text is Bulgarian; code, logs and docs are English.**
- **Never log a password, secret, token or code.**

## Load testing

```bash
k6 run -e BASE_URL=https://<production-url> loadtest/spike.js
```

It ramps from 10 to 500 users in 30 seconds and holds them for 2 minutes. It passes if at least 99.5% of requests succeed and 95% answer within 1 second ([docs/slo.md](docs/slo.md)). Check with DevOps before running it against production. Don't run it against staging: one free instance says nothing about the rush.

## Troubleshooting

| Problem | Fix |
|---|---|
| `pool timed out` at start | PostgreSQL is not reachable. On Windows use `127.0.0.1`, not `localhost`, in `DATABASE_URL` |
| `APP_SIGNING_KEY_PEM is required` | Set it, or use `scripts/dev.sh`, which uses the test-only key |
| Forms answer "Формулярът е изтекъл" | The session expired (30 minutes idle) and its CSRF token with it. Reload the page |
| Staging is slow to answer | Render's free plan sleeps after 15 minutes. The first request wakes it |
| Staging lost every user and module | Render replaces the free database every 30 days. Sync the Blueprint and register again |
