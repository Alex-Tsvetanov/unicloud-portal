---
name: sec-access-control
description: Build and verify the UniCloud RBAC matrix (STUDENT, TEACHER, INSPECTOR, ADMIN, other UniCloud services, anonymous) and check that every endpoint enforces it - including object-level checks against reading or editing other users' profiles and enrollments (IDOR/BOLA), privilege escalation via registration or mass assignment, and IAM/MFA for GitHub and Azure. Use for "RBAC", "who can do what", "access rules", "can a student see another student's profile", "IAM", "MFA", or any new endpoint.
---

# Access control and RBAC

Maintains `docs/security/RBAC-MATRIX.md` and reports access-control findings. Read `AGENTS.md` first.

## 1. The matrix

Rows are resources and actions; columns are principals. Cells: `✓` allowed, `own` only their own records, `taught` only offerings they teach, `✗` denied, `?` undecided (must be resolved by the team, list it).

Principals: Anonymous, STUDENT (ACTIVE), STUDENT (INTERRUPTED/GRADUATED), TEACHER, INSPECTOR, ADMIN, Service (another UniCloud team).

Resources and actions to cover at minimum: register; log in/out; view own profile; edit own profile (which fields: never `role`, `enabled`, `status`, `faculty_number`); view another user's profile; list users; change a user's role; enable/disable an account; change student status; view course catalog; create/edit course; create/edit offering (capacity, open/close); enroll; cancel enrollment; view own enrollments; view enrollment list of an offering; enroll/cancel on behalf of a student; read identity/enrollment data via the service API; `/health` and `/`.

Add one extra column, **Enforced at**, citing where each rule is enforced (`SecurityFilterChain` rule, `@PreAuthorize`, service-layer check with `file:line`), or `not enforced` / `not implemented`.

Proposed defaults (the team confirms them): students `own` for profile and enrollments; INTERRUPTED/GRADUATED students can view but not enroll; teachers see enrollment lists only for offerings they teach; inspectors manage offerings, student status and enrollments on behalf of students; only ADMIN changes roles and enables accounts; services read through a separate, read-only, authenticated API with no write access.

## 2. Verify enforcement in code

1. Inventory every endpoint: `grep -rnE '@(Request|Get|Post|Put|Patch|Delete)Mapping' src/main/java`. Include the method, path and the controller method.
2. Find the security configuration: `SecurityFilterChain` beans, `@EnableMethodSecurity`, `@PreAuthorize`/`@Secured`/`@RolesAllowed`, custom filters. If `spring-boot-starter-security` is not in `pom.xml`, every endpoint is public: record that as the first finding once endpoints beyond `/health` and `/` exist.
3. For each endpoint, check:
   - **Deny by default**: the chain ends with `.anyRequest().authenticated()` (or `denyAll()`), not `permitAll()`.
   - **Role check** matches the matrix. Watch the `ROLE_` prefix: `hasRole("ADMIN")` expects authority `ROLE_ADMIN`.
   - **Object-level check**: any `{id}`, `studentId`, `userId`, `offeringId` from the path, query or body is compared with the authenticated principal (or the principal's teaching assignments) **in the service or query**, e.g. `WHERE student_id = :currentUserId`. Endpoints like `/me` that take the ID from the principal are preferred.
   - **Mass assignment**: request DTOs don't bind `role`, `enabled`, `status`, `id`, `userId`; entities are never bound directly from a request body.
   - **Business rules**: only ACTIVE students enroll (`students.status`), seat locking per README, offering must be open.
   - **Service API**: authenticated (client credentials / API key in a header, not a query string), read-only, returns only the fields the consuming team needs.
4. Check there are tests proving denial: for every `✗` and `own` cell that has an endpoint, a MockMvc/WebMvc test that a different student, a teacher of another course, or an anonymous user gets 401/403/404. Missing denial tests are hardening notes; a reachable missing check is a finding.

## 3. IAM outside the app (needs validation unless the repo shows it)

- GitHub: branch protection on `main`, required reviews and checks, who has admin, 2FA required for collaborators.
- Azure: who has Owner/Contributor on `unicloud-rg`; the deploy identity is Contributor on the resource group only and its federated credential trusts only `repo:<owner>/unicloud-portal:ref:refs/heads/main` (see `infra/setup-azure.sh`); MFA on human Azure accounts.
- MySQL: the app should connect as a least-privilege user (SELECT/INSERT/UPDATE/DELETE on the schema, plus a separate migration user), not as the server admin created by the setup script.

Write each as a precise question for DevOps.

## 4. Output

Update `docs/security/RBAC-MATRIX.md` (header with date and commit SHA, the matrix, the endpoint inventory, the open `?` decisions). Return findings in the shape defined in `sec-sweep` with refs to the matrix row.
