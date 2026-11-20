# PRD: UniCloud Portal, week 1 (Rust rewrite)

Status: accepted design, not built (2026-11-20)
Owner: Team 1, "Portal and Identity"
Sources: the Moodle project brief (MCSE06, "Проект 1: UniCloud"), the week 1 brief from the Rector's office (`01-week`), the other teams' interface sheet (`ServiceInteraction.xlsx`), and [docs/sso.md](sso.md).
Related: [GLOSSARY.md](../GLOSSARY.md), [ADRs](adr/), [design](../openspec/changes/rust-rewrite/design.md), [specs](../openspec/changes/rust-rewrite/specs/), [SLO](slo.md).

## 1. Problem

The old server in the Rector's office failed. Over 4,000 first-year students queued outside the student offices. The university needs one public portal where people sign in, see their academic profile and enroll in elective courses. It must survive the 08:00 rush on enrollment day, cost 0.00 BGN, and from week 2 act as the single source of truth for who is who for the other UniCloud teams.

The portal already exists in Java. We are rebuilding it in Rust (see [ADR 0001](adr/0001-rust-rewrite-with-hypermedia-ui.md)). The rebuild must keep the single sign-on contract that module teams already code against.

## 2. Users

| User | Needs |
|---|---|
| Student | Sign in, see own academic record and status, browse this semester's offerings, enroll while places last, see own enrollments |
| Teacher | See own offerings and the students in them, reject an enrollment |
| Dean | See the students and offerings of their faculty |
| Inspector | Create students, manage courses and offerings |
| Admin | Create staff accounts, register modules for single sign-on |
| Module team (Teams 2 to 5) | Sign users in through the portal, read who is who and who is enrolled where |
| Academic Council | A measurable, honest promise about uptime and speed |

## 3. Goals

1. **G1. A live portal from any device.** Public `https` address, Bulgarian UI, usable on a phone.
2. **G2. Accounts and sign-in** for students, teachers, deans, inspectors and admins, with their university data (faculty number, specialty, study year, role).
3. **G3. Academic profile** with student status: Active, Interrupted or Graduated.
4. **G4. Catalogue and enrollment.** This semester's active offerings. A student request takes a free place at once and never oversells.
5. **G5. Clear role boundaries.** What a student sees differs from what a teacher, dean or inspector can manage.
6. **G6. Survive the 08:00 rush.** No single point of failure. Load spreads across replicas. A live demo shows it.
7. **G7. A written promise** with measurable indicators, an availability target, the monthly downtime it allows, and what happens when that is used up.
8. **G8. Single sign-on for every module**, compatible with `docs/sso.md` sections 1 to 10.
9. **G9. A directory for modules.** Read-only lookups that match the interface sheet: the user's role by user id, then name, faculty number, faculty and enrolled courses for a student; name and courses for a teacher; name and faculty for a dean.

## 4. Non-goals (week 1)

- Grades. Team 3 owns them. Learning materials. Team 2 owns them.
- Public self sign-up. Accounts are created by inspectors and admins.
- Email or SMS. There is no budget for sending them.
- Single logout across modules, `private_key_jwt`, key rotation, refresh tokens.
- Migrating data from the Java portal. The cutover starts with a fresh database.
- Paid infrastructure of any kind.

## 5. Requirements

Each line links to the OpenSpec capability that pins it.

| # | Requirement | Spec |
|---|---|---|
| R1 | An inspector creates a student with faculty number, specialty, study year and status. An admin creates a teacher, dean, inspector or admin. The creator sets a first password, and the user must change it at first sign-in. | accounts-and-sign-in |
| R2 | A user signs in with email and password and signs out. One cookie session serves the portal UI and single sign-on. | accounts-and-sign-in |
| R3 | A student sees their academic record and status. | academic-profile |
| R4 | Anyone signed in sees this semester's open offerings with free places. | course-catalog |
| R5 | An inspector manages faculties, specialties, courses and offerings. | course-catalog |
| R6 | An Active student enrolls in an open offering only while places remain. Two students never take the last place together. | enrollment |
| R7 | The teacher of an offering rejects an enrollment. The place is freed. | enrollment |
| R8 | Permissions decide every action. Each role has defaults, and a user's set may differ. Deans see only their own faculty. | role-access |
| R9 | The portal is an OIDC provider that matches `docs/sso.md`: discovery, authorize with PKCE, token, userinfo, logout, and the callback rules per environment. | sso-provider |
| R10 | New contract additions: role `DEAN`, the `faculty` claim and permission `STUDENT_PROFILE_READ_FACULTY`. Existing names do not change. | sso-provider, role-access |
| R11 | Modules read the directory with a `client_credentials` token that has the `directory` scope. | module-directory |
| R12 | Staging has self-service tools for module teams: test users with any permissions, and module registration. | staging-tools |
| R13 | `/health`, the replica name, graceful shutdown, at least 2 production replicas, and an SLO. | operations |

## 6. Success measures

The targets come from [docs/slo.md](slo.md):

- Availability of sign-in, profile and enrollment of at least 99.5% per calendar month.
- 95% of profile and catalogue pages served in under 1 second.
- At least 99.5% of sign-in and enrollment attempts succeed, not counting the user's own mistakes.
- The k6 spike test (10 to 500 users in 30 seconds, then held for 2 minutes) passes against production.
- Every module team that followed `docs/sso.md` signs in against the Rust staging without changing code, after registering again.

## 7. Constraints

- 0.00 BGN: free plans and student credits only.
- Staging is the fork's Render Blueprint (`render.yaml`): one free web service that sleeps after 15 minutes, and a free PostgreSQL that expires after 30 days.
- Production is Azure Container Apps on student credits, with 2 to 10 replicas and Azure PostgreSQL Flexible Server.
- The UI is Bulgarian. Code, APIs and docs are English.

## 8. Risks

| Risk | Mitigation |
|---|---|
| A hand-built OIDC provider has a security gap | Black-box conformance suite ported from the Java tests and checked against a recording of the Java staging. Run the OpenID Foundation tests if time allows. |
| Rust builds on Render's free builder are slow or run out of memory | cargo-chef dependency caching. The fallback is a prebuilt image, which needs a new Render service and a new address. |
| Modules hard-code the four roles and break on `DEAN` | `docs/sso.md` warns that unknown roles mean no access. Tell the teams before the release. |
| Staging's free database expires every 30 days | Migrations and seed data rebuild it on start. Module teams register again. |
| The interface sheet changes when teams 2 to 5 see their own briefs | The directory API is versioned under `/api/v1`. Only add, never rename. |
