# Tasks

## 0. Before any Rust
- [x] 0.1 Rebase `rust-rewrite` onto the fork's `main` (it has `render.yaml`)
- [ ] 0.1b Merge the `sso.md` notice on unknown roles and the coming `DEAN` (branch `docs/sso-role-values`) into the fork's `main` (the team repo comes later)
- [ ] 0.2 Record the Java staging: discovery, JWKS, token response, decoded tokens, error bodies. Save under `tests/golden/`
- [ ] 0.3 Confirm the "Unverified (Spring default)" items in `docs/sso-wire-contract.md` against the recording

## 1. Skeleton
- [ ] 1.1 Cargo project, config, tracing, `/health`, `/`, `X-Served-By`, graceful shutdown
- [ ] 1.2 sqlx pool, migrations at start, seed data (faculties, specialties, sample courses)
- [ ] 1.3 Dockerfile with cargo-chef. Measure the Render build time and memory
- [ ] 1.4 CI: fmt, clippy, sqlx prepare check, tests on PostgreSQL 17, Docker build

## 2. Accounts and access
- [ ] 2.1 Role, Permission bits 0 to 10, role defaults, permission checks
- [ ] 2.2 Argon2id hashing, sign-in, sign-out, session cookie, CSRF
- [ ] 2.3 First-password change
- [ ] 2.4 Inspector creates students. Admin creates staff

## 3. Academic data and enrollment
- [ ] 3.1 Profile page with student status
- [ ] 3.2 Catalogue: courses and offerings, inspector management
- [ ] 3.3 Enroll, cancel, teacher reject, seat accounting
- [ ] 3.4 `enrollment_race` test
- [ ] 3.5 Teacher and dean views

## 4. OIDC provider
- [ ] 4.1 Signing key from PEM, JWKS, RFC 7638 `kid`, discovery
- [ ] 4.2 Client registration (admin page), secret hashing, callback policy and host list
- [ ] 4.3 Authorize with PKCE, login round trip, codes
- [ ] 4.4 Token endpoint, single-use codes, replay revocation
- [ ] 4.5 ID token and access token claims, `/userinfo`
- [ ] 4.6 `/connect/logout`, 24 hour grant cleanup
- [ ] 4.7 `sso_conformance` suite green against the golden recording

## 5. Directory and staging
- [ ] 5.1 `client_credentials` grant with the `directory` scope
- [ ] 5.2 Directory endpoints and pagination
- [ ] 5.3 Staging page and API, including faculty and the `directory` grant

## 6. Docs and release
- [ ] 6.1 `docs/sso.md`: `DEAN`, `faculty`, unknown-role rule, directory section, section 11 rewrite, corrections
- [ ] 6.2 README, `render.yaml`, `infra/setup-azure.sh`, deploy job
- [ ] 6.3 Remove the Java code and MySQL migrations
- [ ] 6.4 Focused security review of `oidc/` and `accounts/`
- [ ] 6.5 Spike test against production, results in `docs/slo.md`
