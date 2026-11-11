# AGENTS.md

Instructions for coding agents (Claude Code, Codex, Cursor, Copilot and others) working in this repository. Humans should start with [README.md](README.md); this file adds what an agent needs and, in particular, how the **Security Engineer** responsibilities for UniCloud are run as repeatable agent tasks.

## Project in one paragraph

UniCloud Portal is the university's single portal for accounts, academic profiles and course enrollment, and the source of truth for "who is who" for the other UniCloud teams. Stack: Java 25, Spring Boot 4.1, MySQL 8.4 (Azure Flexible Server), Flyway, Docker, GitHub Actions (OIDC to Azure), Azure Container Apps (2 to 10 replicas), images in GHCR. Every merge to `main` goes to production with no staging, so security checks have to run before merge.

## Ground rules for every agent

- Follow the rules in the README section "Rules the code must follow". Never weaken them to fix a security issue; find a fix that keeps them.
- **Never** put credentials, tokens, connection strings with passwords, or real personal data in code, tests, fixtures, docs, commit messages or reports. Use obviously fake values (`student1@example.test`).
- **Never** probe, load test or scan the production portal, the Azure subscription, the production database or GHCR. Security work here is source review plus local tests against the local Docker MySQL. Anything that needs production evidence is written up as "needs validation by DevOps", with the exact fact to check.
- Schema changes go in a new Flyway migration (`V<n>__...sql`), never by editing an applied one.
- Run `./mvnw verify` (local MySQL running) before proposing a change.

## Security Engineer: responsibilities as agent skills

The Security Engineer role (Алекс) covers seven general responsibilities and seven UniCloud-specific checks. Each is implemented as a project skill in `.claude/skills/` so any agent can run it the same way every time. Skills are plain Markdown; agents that don't load skills automatically should read the `SKILL.md` file and follow it.

| Skill | Responsibility (from the role description) | Deliverable it maintains |
| --- | --- | --- |
| `sec-sweep` | Runs the others in order; the periodic entry point | `docs/security/reports/<date>-security-test-report.md` |
| `sec-threat-model` | 1. Design security architecture · Threat Model | `docs/security/THREAT-MODEL.md` |
| `sec-requirements` | 1, 6 · Security Requirements | `docs/security/SECURITY-REQUIREMENTS.md` |
| `sec-access-control` | 2. Manage access controls (IAM, MFA, RBAC) · access rules for students, teachers, administration · access to other users' profiles | `docs/security/RBAC-MATRIX.md` |
| `sec-auth-review` | Registration, authentication and password storage | findings in the vulnerability assessment |
| `sec-vuln-assessment` | 3. Vulnerability assessments · SQL injection, XSS, misconfiguration, dependencies | `docs/security/VULNERABILITY-ASSESSMENT.md` |
| `sec-data-protection` | 6. Compliance (GDPR) · personal data protection and secrets management | GDPR and secrets sections of the requirements and assessment |
| `sec-devsecops` | 4. Automate security workflows · 7. Collaborate and train (DevSecOps) | `.github/workflows/security.yml`, PR checklist |
| `sec-monitoring` | 5. Monitor and analyze threats, incident response | `docs/security/MONITORING-AND-IR.md` |
| `security-audit` (third party, Cloudflare) | Deep, evidence-based vulnerability hunt across the whole repo | run directory outside the repo; summary in the assessment |

IDS/IPS, VPN, SIEM and MFA are part of the general role, but they are only in scope when the architecture needs them. The skills record them as requirements or "not applicable yet" with a reason, rather than inventing infrastructure.

The `security-audit` skill is vendored from [cloudflare/security-audit-skill](https://github.com/cloudflare/security-audit-skill) and pinned in `skills-lock.json`. Don't edit its files; update it with `npx skills update security-audit` and review the diff. It writes its run directories outside the repo (`~/security-audit-skill/unicloud-portal/run-<N>`) and never runs target code without an OS sandbox. The ID scheme (V-xx) and disclosure rule in this file apply to anything imported from it.

### When to run what

| Trigger | Run |
| --- | --- |
| Pull request that touches auth, roles, controllers, SQL, migrations, `pom.xml`, `Dockerfile`, workflows or `infra/` | `sec-sweep --scope diff` (it picks the relevant skills) |
| Weekly (Monday morning), or as a scheduled routine | `sec-sweep` (full, incremental against the last sweep) |
| New feature or new endpoint designed | `sec-threat-model`, then `sec-access-control` |
| Before each enrollment campaign and at the end of each sprint/week | `sec-sweep --deep` (includes a `security-audit` run) |
| Suspected incident | `sec-monitoring` (incident section) |

Automated scanners (CodeQL, Trivy, dependency review) also run in GitHub Actions on every pull request and weekly; see `.github/workflows/security.yml`. The agent sweep reads their results instead of re-implementing them.

How to run a sweep from a terminal, non-interactively:

```bash
claude -p "/sec-sweep"
```

### Where results go

```text
docs/security/
  SECURITY-REQUIREMENTS.md     living document, numbered SR-xx requirements
  THREAT-MODEL.md              living document, STRIDE per trust boundary, numbered T-xx threats
  RBAC-MATRIX.md               living document, role x resource x action, with enforcement points
  VULNERABILITY-ASSESSMENT.md  living register of findings (V-xx) with status
  MONITORING-AND-IR.md         security events, alert queries, incident runbook
  reports/YYYY-MM-DD-security-test-report.md   one per sweep, never edited afterwards
  .last-sweep                  commit SHA the last full sweep reviewed
```

Living documents are updated in place and keep stable IDs (`SR-07`, `T-12`, `V-03`): never renumber, mark retired items as retired. Reports are dated snapshots.

### Evidence bar (shared by all security skills)

- A **finding** names who the attacker is, what they send, which control should stop them, what boundary is crossed and what they get. Cite `file:line`.
- A missing best practice with no reachable impact is a **hardening note**, not a vulnerability. Severity comes only with demonstrated impact (critical / high / medium / low / info, as defined in `.claude/skills/security-audit/SKILL.md`).
- When the decisive fact lives outside the repo (Azure settings, GitHub branch protection, secrets), the status is **needs validation** with the exact question for DevOps.
- Prefer the smallest fix at the last trusted decision point, plus a regression test.

### Disclosure

This repository is public. A **confirmed high or critical** finding in code that is already deployed must not be written up in detail in a commit, PR or issue until it is fixed: report it privately (GitHub → Security → Advisories → "Report a vulnerability", or directly to the team lead and DevOps), and record only "V-xx: under private remediation" in the register. Everything else goes in the docs as normal.

### Language

Deliverables for the Academic Council and the team (`docs/security/*`) are written in Bulgarian unless the team asks otherwise; IDs, code, file paths and technical terms stay in English. Skill files and this document stay in English so every agent reads them the same way.
