---
name: sec-threat-model
description: Create or update the UniCloud threat model (STRIDE per trust boundary, data-flow diagram, assets, abuse cases) and the security-architecture decisions behind it (firewalling/ingress, TLS and cryptography, network exposure, IDS/IPS, VPN, SIEM - in scope only when the architecture needs them). Use for "threat model", "security architecture", "what can go wrong", a new feature, endpoint, integration or consuming team, or when infra/workflows change.
---

# Threat model and security architecture

Maintains `docs/security/THREAT-MODEL.md`. Read `AGENTS.md` first.

## Inputs to read every time

- `README.md` (architecture, pipeline, rules), `AGENTS.md`
- `src/main/java/**`, `src/main/resources/application*.properties`, `db/migration/*.sql`
- `Dockerfile`, `.github/workflows/*.yml`, `infra/setup-azure.sh`
- `docs/security/*.md` (existing IDs), the latest report in `docs/security/reports/`

## Model to maintain

### Assets (rank by impact)

At minimum: user accounts and credentials (password hashes, sessions/tokens); student personal and academic data (name, email, faculty number, specialty, year, status); enrollment records and seat capacity (fairness at 08:00); the role assignments themselves (`users.role`), since other UniCloud teams trust this portal as the source of truth for identity and enrollment; deployment credentials (`AZURE_*` OIDC identity, `GHCR_PAT`, `DATABASE_URL`); availability during the enrollment rush.

### Actors

Anonymous internet user; registered student (ACTIVE / INTERRUPTED / GRADUATED); teacher; inspector (учебен отдел); admin; another UniCloud team's service calling the API; a contributor opening a pull request (including from a fork); DevOps with Azure access; a compromised dependency or GitHub Action.

### Trust boundaries (draw them in a Mermaid flowchart)

1. Internet → Azure Container Apps ingress (TLS termination, replicas)
2. App → MySQL Flexible Server (credentials in `DATABASE_URL`, TLS, network access rule)
3. Other UniCloud services → portal API (service authentication, what they may read)
4. Role boundaries inside the app: student ↔ other student, student ↔ teacher, teacher ↔ inspector/admin
5. GitHub pull request → Actions runner → GHCR → Azure (OIDC subject, permissions, secrets exposure)
6. Developer machine / Codespace → repository (secrets in commits)

### Threats

For each boundary, walk STRIDE and write each credible threat as a row:

| ID | Boundary | STRIDE | Threat (actor → action → impact) | Existing control (file:line or "none yet") | Requirement (SR-xx) | Residual risk |

Always cover these UniCloud-specific abuse cases:

- A student reads or edits another student's profile or enrollments by changing an ID (IDOR/BOLA).
- A student self-registers as TEACHER/ADMIN (mass assignment of `role`), or a teacher grants themself INSPECTOR.
- Registration with someone else's faculty number or email; account enumeration through registration or login errors.
- Credential stuffing and password spraying, timed for the 08:00 rush when monitoring is noisiest.
- Seat races: two requests take the last seat; one student scripts hundreds of enroll/cancel calls to hold seats.
- INTERRUPTED or GRADUATED student enrolls anyway.
- A downstream team trusts stale or forged identity data.
- A pull request from a fork exfiltrates secrets or pushes an image; a compromised Action or Maven dependency.
- The database is reachable from any Azure tenant because of the firewall rule; leaked `DATABASE_URL` gives admin access.
- Personal data leaks through logs, error pages, the `/` endpoint, or exports to other teams.
- Denial of service at 08:00 (expensive unauthenticated endpoints, connection-pool exhaustion: 10 replicas × 15 = 150 of 171 connections).

### Security architecture decisions

Record each as a short decision with status (`in place`, `required`, `not applicable yet` + reason):
ingress and TLS (HTTPS only, HSTS, minimum TLS version), network exposure of MySQL (public access rule vs. VNet/private endpoint, and what the free tier allows), WAF/rate limiting at the edge vs. in-app, cryptography (password hashing algorithm and cost, token signing, TLS to MySQL with server verification), secret storage (GitHub secrets vs. Container Apps secrets vs. Key Vault), least-privilege DB user for the app vs. the admin user, IDS/IPS, VPN, SIEM, MFA (for admins/inspectors, and for GitHub/Azure accounts). Don't propose paid services: the budget is 0.00 лв. Say what the free tier allows.

## Procedure

1. If the file doesn't exist, build the model from scratch. Otherwise check for drift: new endpoints, entities, workflows, settings or integrations since the date in the file header, and add or update threats. Never renumber; mark obsolete threats `retired`.
2. Each threat cites the control that mitigates it with `file:line`, or "none yet" plus the SR-xx requirement that should. If a requirement is missing, hand it to `sec-requirements`.
3. Rate residual risk as likelihood × impact (low/medium/high) given existing controls, not the worst case without them.
4. Update the header: date, commit SHA, author ("security-engineer agent" plus the human reviewer).
5. Return new or changed T-xx rows and new requirements needed.
