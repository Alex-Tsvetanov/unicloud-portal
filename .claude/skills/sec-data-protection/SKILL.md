---
name: sec-data-protection
description: Check UniCloud credentials management (DATABASE_URL, Azure OIDC, GHCR_PAT, default and test credentials, git history, images) and personal-data protection under GDPR (inventory of personal data, minimization, lawful basis, retention, data-subject rights, logging, sharing with other UniCloud teams). Use for "secrets", "credentials", "leaked password", "GDPR", "personal data", "privacy", "лични данни", or when migrations, logging, properties, workflows or infra change.
---

# Credentials management and personal-data protection

Feeds the credentials and GDPR sections of `docs/security/SECURITY-REQUIREMENTS.md` and findings into the register. Read `AGENTS.md` first. Never copy a credential value into any output; refer to it by location and redact as `***`.

## Part A: credentials

1. **Working tree and history.** If `gitleaks` or `trivy` is installed: `gitleaks detect --redact -v` (history) / `trivy fs --scanners secret .`. Otherwise:
   ```bash
   git grep -nIE '(password|passwd|pwd|secret|token|api[_-]?key|client[_-]?secret)\s*[=:]\s*[^$\s{][^\s]{5,}' -- . ':!*.md'
   git log -p --all -S 'password=' --oneline | head -50
   git grep -nE 'ghp_|gho_|github_pat_|AKIA|-----BEGIN (RSA|EC|OPENSSH|PRIVATE)' $(git rev-list --all | head -200)
   ```
   Known, intentional dev-only values (`password=dev` local Docker default in `application.properties`, `password=test` for the CI service container) are acceptable only while they match nothing in production. Record that as a passed check with the condition, and flag it if the production `DATABASE_URL` is ever unset (the app would silently try the local default).
2. **How production credentials flow.** `infra/setup-azure.sh` → printed once → GitHub Actions secret `DATABASE_URL` → Container Apps secret `db-url` → env var. Check: values never echoed in workflow logs (`set -x`, `echo $VAR`), passed as `--secrets`/`secretref` rather than plain `--env-vars`, and not baked into images. The DB URL embeds the **admin** user and password: record a least-privilege app user and rotation as requirements.
3. **CI tokens.** `GHCR_PAT` scope should be `read:packages` only, owned by a bot or the repo owner, with an expiry. `GITHUB_TOKEN` permissions minimal per job. The Azure identity uses OIDC (no stored password) and its federated subject is limited to `main`. Items you can't see from the repo are `needs-validation` questions for DevOps.
4. **App-level keys (when they appear).** JWT signing keys, reset-token keys, service API keys for other UniCloud teams: from environment or a secret store, ≥ 256 bits, rotatable, different per environment, never in `application.properties`.
5. **After any exposure:** rotate first, then clean history; a credential that reached a public repo is compromised regardless of history rewriting.

## Part B: personal data and GDPR

1. **Inventory.** From migrations and entities, list every personal-data field (`users.email`, `first_name`, `last_name`, `students.faculty_number`, `specialty`, `study_year`, `status`, enrollments, timestamps, IP addresses in logs). Per field: purpose, lawful basis (for a state university most are Art. 6(1)(e) public task or (c) legal obligation; consent is usually the wrong basis here), who can see it (link to the RBAC matrix), which UniCloud teams receive it, retention.
2. **Minimization.** Flag new columns without a stated purpose (e.g., ЕГН, phone, address, date of birth). ЕГН in particular needs strong justification and extra protection; recommend not storing it.
3. **Data-subject rights.** Access/export (Art. 15), rectification (Art. 16), and the retention rule for GRADUATED/INTERRUPTED students. Erasure is limited by the university's legal record-keeping duties: write down which data must be kept and which can go.
4. **Leakage paths.** Personal data in logs (`log.info("... {}", user)`, entity `toString()`), error responses, the `/` and `/health` endpoints, analytics, load-test scripts, test fixtures (use `example.test` addresses), and over-broad service API responses to other teams.
5. **Security of processing (Art. 32).** TLS in transit (including to MySQL with certificate verification), encryption at rest (Azure MySQL default: needs-validation), access logging for staff reads of student records, backup retention (Azure default 7 days: needs-validation).
6. **Breach readiness (Art. 33).** The 72-hour notification path (DPO, КЗЛД) is described in `docs/security/MONITORING-AND-IR.md`.

## Output

Findings in the `sec-sweep` shape. GDPR gaps without a reachable exploit are requirements (`not met`) or hardening notes, not vulnerabilities. Update the personal-data inventory table in `docs/security/SECURITY-REQUIREMENTS.md`.
