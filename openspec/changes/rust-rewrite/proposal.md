# Change: Rebuild the portal in Rust for week 1

## Why
The week 1 brief asks for a live portal with accounts, academic profiles, a catalogue with enrollment, role boundaries, survival of the 08:00 rush and a written uptime promise. From week 2 the portal must also be the single source of truth for the other teams. The team decided to rebuild the Java portal in Rust, with a hypermedia UI, while keeping the single sign-on contract that module teams already use ([ADR 0001](../../../docs/adr/0001-rust-rewrite-with-hypermedia-ui.md)). The other teams' interface sheet needs data the current contract cannot give: deans, faculties and lookups with no user present ([ADR 0002](../../../docs/adr/0002-grow-the-sso-contract.md)).

## What changes
- **BREAKING (internal only):** the Java/Spring code, its MySQL migrations and the portal's own JWT API (`/api/v1/auth/login`, `/api/v1/auth/sso-session`, Bearer tokens with `aud=unicloud-api`) are removed. One cookie session replaces them ([ADR 0003](../../../docs/adr/0003-one-cookie-session.md)). No module used them.
- **BREAKING (operations):** fresh PostgreSQL database at cutover. Module registrations and staging users must be created again.
- New Rust service with a hand-built OIDC provider that matches `docs/sso.md` sections 1 to 10.
- Added to the contract: role `DEAN`, permission `STUDENT_PROFILE_READ_FACULTY` (bit 10), the `faculty` claim, the `client_credentials` grant with the `directory` scope, and a read-only directory API.
- Faculties and specialties become entities. Teachers, deans and inspectors belong to one faculty.
- Enrollment: a request takes a free place at once. The teacher may reject it, which frees the place.
- Accounts are created by inspectors (students) and admins (staff), with a first password the user must change.
- Admins register modules on a portal page instead of a Bearer API.
- Bulgarian server-rendered UI with Maud components and htmx.
- `docs/slo.md` states the uptime promise.

## Impact
- Specs (all new): `accounts-and-sign-in`, `academic-profile`, `course-catalog`, `enrollment`, `role-access`, `sso-provider`, `module-directory`, `staging-tools`, `operations`.
- Docs: `docs/sso.md` (additions, section 11 rewritten, corrections), `README.md`, `render.yaml`, `.github/workflows/deploy.yml`, `Dockerfile`, `infra/setup-azure.sh`.
- Module teams: register again after cutover. Treat unknown roles as no access. Optionally adopt the directory API.
