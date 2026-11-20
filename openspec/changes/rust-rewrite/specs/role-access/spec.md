## ADDED Requirements

### Requirement: Roles
Every user SHALL have exactly one role: `STUDENT`, `TEACHER`, `DEAN`, `INSPECTOR` or `ADMIN`. These names MUST be used unchanged on the wire.

#### Scenario: Role in claims
- **WHEN** a dean logs in to a module with scope `unicloud`
- **THEN** the `role` claim is `"DEAN"`

### Requirement: Permissions and bits
The portal SHALL know exactly these permissions, each at a fixed bit that MUST never change: 0 `STAFF_CREATE`, 1 `STUDENT_CREATE`, 2 `STUDENT_PROFILE_READ_OWN`, 3 `STUDENT_PROFILE_READ_ANY`, 4 `STUDENT_PROFILE_READ_ENROLLED`, 5 `COURSE_CREATE_MODIFY`, 6 `COURSE_OFFERING_READ`, 7 `COURSE_OFFERING_CREATE_MODIFY`, 8 `ENROLLMENT_REQUEST_CREATE`, 9 `ENROLLMENT_REQUEST_ACCEPT_REJECT`, 10 `STUDENT_PROFILE_READ_FACULTY`. Lists of permissions on the wire SHALL be arrays of names in bit order.

#### Scenario: Permission list order
- **WHEN** a user holds `ENROLLMENT_REQUEST_CREATE` and `STUDENT_PROFILE_READ_OWN`
- **THEN** the `permissions` claim is `["STUDENT_PROFILE_READ_OWN", "ENROLLMENT_REQUEST_CREATE"]`

### Requirement: Role defaults
A new user SHALL get the default permissions of their role. `ADMIN`: `STAFF_CREATE`. `INSPECTOR`: `STUDENT_CREATE`, `STUDENT_PROFILE_READ_ANY`, `COURSE_CREATE_MODIFY`, `COURSE_OFFERING_READ`, `COURSE_OFFERING_CREATE_MODIFY`. `TEACHER`: `COURSE_OFFERING_READ`, `STUDENT_PROFILE_READ_ENROLLED`, `ENROLLMENT_REQUEST_ACCEPT_REJECT`. `STUDENT`: `STUDENT_PROFILE_READ_OWN`, `COURSE_OFFERING_READ`, `ENROLLMENT_REQUEST_CREATE`. `DEAN`: `STUDENT_PROFILE_READ_FACULTY`, `COURSE_OFFERING_READ`.

#### Scenario: New teacher
- **WHEN** an admin creates a teacher
- **THEN** the teacher holds exactly the three `TEACHER` defaults

### Requirement: Permissions decide access
Every action SHALL be allowed or refused by the user's permissions, not by the role name. A user's set MAY differ from the role's defaults.

#### Scenario: Student given an extra permission
- **WHEN** a student is given `COURSE_CREATE_MODIFY` on staging
- **THEN** that student can create courses

### Requirement: One dean per faculty
A faculty SHALL have at most one user with role `DEAN`.

#### Scenario: Second dean
- **WHEN** an admin creates a second dean for FCST
- **THEN** the portal refuses with a Bulgarian message

### Requirement: Role-specific views
The portal navigation and pages SHALL show only what the user's permissions allow. A student sees profile, catalogue and own enrollments. A teacher sees profile, catalogue and own offerings with rosters. A dean sees the faculty's offerings and students. An inspector sees student and course management. An admin sees staff management and module registration.

#### Scenario: Student navigation
- **WHEN** a student opens any page
- **THEN** the navigation has no links to staff, module or course management

#### Scenario: Direct URL without permission
- **WHEN** a student calls `/api/v1/portal/admin/modules` directly
- **THEN** the portal answers 403

### Requirement: Unknown roles in modules
The module guide (`docs/sso.md`) SHALL tell module teams that `role` may gain new values and that a module MUST treat an unknown role as no access.

#### Scenario: Guide states the rule
- **WHEN** a module team reads the User claims section of `docs/sso.md`
- **THEN** it states that new roles may appear and unknown roles mean no access
