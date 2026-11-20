# Grow the SSO contract instead of renaming it

Status: accepted (2026-11-20)

The other teams' interface sheet (ServiceInteraction.xlsx) speaks of Professor, Dean, Faculty and FacultyID. The published contract in `docs/sso.md` fixes `role` to `STUDENT | TEACHER | INSPECTOR | ADMIN` and has no faculty. We keep every published name and only add to it, so no module that follows `docs/sso.md` breaks:

- `TEACHER` is the professor. We do not rename it.
- A new role, `DEAN`, and a new permission, `STUDENT_PROFILE_READ_FACULTY`, at the next free bit. The existing ten permissions keep their names and bits.
- A new `faculty` claim (`{"code", "name"}`) in the `unicloud` scope.
- A read-only directory API under `/api/v1/directory`. Modules call it with a `client_credentials` token that carries the `directory` scope. A module token used to work only on `/userinfo`. The Excel's lookups by user, course and faculty need data with no user present.
- The shared course identifier is the course code (for example `MATH1`), not a database id.

Grades stay with Team 3 and materials with Team 2. The portal does not store or relay them.
