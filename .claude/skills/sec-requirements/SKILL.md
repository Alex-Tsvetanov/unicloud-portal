---
name: sec-requirements
description: Maintain the UniCloud Security Requirements (numbered SR-xx, each testable, with a verification method and current status), covering authentication, access control, input handling, data protection and GDPR, secrets, logging, availability and the CI/CD pipeline. Use for "security requirements", "what must the portal guarantee", acceptance criteria for a feature, or to check which requirements are met.
---

# Security requirements

Maintains `docs/security/SECURITY-REQUIREMENTS.md`. Read `AGENTS.md` first.

## Format

One table per area. Every requirement must be testable: if you can't say how to check it, rewrite it.

| ID | Requirement | Priority (MUST/SHOULD/MAY) | Verification (test, config check, review) | Status | Evidence |

Status is one of `met` (cite `file:line` or test name), `partially met`, `not met`, `not applicable yet` (feature doesn't exist; give the trigger that makes it apply), `needs validation` (fact outside the repo; name who can confirm).

## Areas and baseline requirements

Start from this baseline and adapt it to what the code actually does. Keep IDs stable once published.

- **Authentication** — passwords hashed with BCrypt (cost ≥ 12) or Argon2id, never logged or returned; minimum length 12 and a breached-password/common-password check; generic login and registration errors (no account enumeration); rate limiting / lockout on login and registration per account and per IP; session cookies `HttpOnly`, `Secure`, `SameSite`; session fixation protection; logout invalidates the session/token; password reset tokens single-use and short-lived; MFA for ADMIN and INSPECTOR (SHOULD, may be "not applicable yet" in week 1).
- **Registration** — users cannot choose their role (role is server-assigned; STUDENT by default, staff roles granted only by ADMIN); faculty number and email uniqueness enforced in the DB (`uq_students_faculty_number`, `uq_users_email`) and by the service; email ownership verified, or university directory check, before the account is enabled.
- **Authorization** — deny by default (every endpoint needs an explicit rule); object-level checks on every ID in a path or body (a student can only reach their own profile and enrollments); RBAC matches `docs/security/RBAC-MATRIX.md`; only ACTIVE students enroll; enrollment takes a seat in one locked transaction (README rule).
- **Input and output** — all SQL parameterized (`JdbcTemplate` `?` placeholders, JPA parameters; no string concatenation); request DTOs validated with Bean Validation; responses are JSON with correct `Content-Type`; any HTML output escaped; CSRF protection for cookie-based sessions; CORS restricted to known UniCloud origins.
- **Data protection and GDPR** — data minimization (store only what the portal needs); TLS everywhere including app→MySQL (`sslMode=VERIFY_IDENTITY` preferred over `REQUIRED`); personal data never in logs, error messages or the `/` and `/health` endpoints; documented retention for graduated/interrupted students; a way to export and correct a user's data (GDPR Art. 15/16); a record of what personal data other UniCloud teams receive and why.
- **Secrets** — no secrets in the repo or images; app uses a least-privilege DB user, not the server admin; secrets rotated after any exposure; `GHCR_PAT` scoped to `read:packages` only.
- **Logging and monitoring** — security events logged (see `sec-monitoring`) with user ID, never password or token; logs retained per policy.
- **Availability** — rate limits that keep one client from starving others at 08:00; pool limits per README.
- **Pipeline** — PRs from forks get no secrets and no write tokens; actions pinned to commit SHAs; deploy identity limited to the resource group and to `main`; security scans required before merge.

## Procedure

1. Read the current file (if any), `docs/security/THREAT-MODEL.md`, and the code in scope.
2. Add requirements that new threats (T-xx without an SR-xx) need. Link each requirement to the threats it mitigates.
3. Re-verify the status of every requirement whose evidence files changed. A requirement is `met` only with evidence you can cite now.
4. Return: new/changed SR-xx, and every MUST that is `not met` (these are candidates for the vulnerability register when the code that needs them exists).
