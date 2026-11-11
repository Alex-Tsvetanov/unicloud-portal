## What and why

<!-- One or two sentences. Link the issue or week brief. -->

## How it was tested

<!-- ./mvnw verify, new tests, manual checks against the local MySQL. -->

## Security checklist

Tick what applies; write "n/a" next to items that don't. See AGENTS.md and docs/security/.

- [ ] Every new endpoint has an access rule matching `docs/security/RBAC-MATRIX.md` and a test that another student (or an anonymous user) gets 401/403/404.
- [ ] IDs from the request are checked against the logged-in user, or taken from the principal (`/me`).
- [ ] Request DTOs don't accept `role`, `enabled`, `status`, `id` or `userId`.
- [ ] SQL uses `?` or named parameters only; no string concatenation.
- [ ] No passwords, tokens, connection strings or real personal data in code, tests, logs or this PR.
- [ ] Responses contain only the personal data the caller needs; new personal-data columns have a row in the inventory in `docs/security/SECURITY-REQUIREMENTS.md`.
- [ ] New dependencies are needed and maintained; the `dependency-review` check is green.
- [ ] Migrations are backward compatible (old and new replicas run side by side).
