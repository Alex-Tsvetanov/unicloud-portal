## ADDED Requirements

### Requirement: Staging only
The staging tools SHALL exist only when the portal runs with staging mode on. Without it, `/staging` and `/api/v1/staging/**` MUST answer 404 to everyone, admins included. Staging mode SHALL also set the callback mode to `any`.

#### Scenario: Production
- **WHEN** an admin requests `GET /api/v1/staging/users` on production
- **THEN** the answer is 404

### Requirement: Staging page
`GET /staging` SHALL serve a page, with `Cache-Control: no-store` and no sign-in, that lets a module team register a module (optionally with the `directory` grant), create a test user with any role, faculty and permission set, and edit a user's permissions. The page SHALL use htmx fragments for the users table and the forms.

#### Scenario: Edit permissions inline
- **WHEN** a team presses "Edit permissions" in the users table and saves
- **THEN** only that row is replaced, without a full page load

### Requirement: Staging API
The staging API SHALL keep the Java routes and shapes, with no sign-in: `GET /api/v1/staging/roles` (`{"roleDefaults": {...}, "permissions": [...]}`, roles in the order `ADMIN, INSPECTOR, TEACHER, DEAN, STUDENT`, lists in bit order); `GET /api/v1/staging/users` (array ordered by id of `{id, email, firstName, lastName, role, faculty, permissions, student}`, `student` a boolean, never a password); `POST /api/v1/staging/users` (201); `PUT /api/v1/staging/users/{id}/permissions` (200, replaces the set exactly, 404 for an unknown id); `POST /api/v1/staging/sso-clients` (201, same body and answer as before, plus an optional boolean `directory`).

#### Scenario: Roles listing
- **WHEN** a team calls `GET /api/v1/staging/roles`
- **THEN** `permissions` has 11 names and `roleDefaults.DEAN` is `["COURSE_OFFERING_READ", "STUDENT_PROFILE_READ_FACULTY"]`

#### Scenario: Register a module from the API
- **WHEN** a team posts `{"clientId": "module-x", "name": "X", "redirectUris": ["http://localhost:3000/cb"]}`
- **THEN** the answer is 201 with a 43-character `clientSecret`

### Requirement: Creating staging users
`POST /api/v1/staging/users` SHALL take `email` (at most 254, stored lower-cased), `password` (8 to 72), `firstName` and `lastName` (1 to 100), `role` (case-insensitive), optional `faculty` (a faculty code, required for teachers, deans and inspectors), optional `permissions` (absent means the role's defaults, `[]` means none) and optional `student` (`facultyNumber` at most 20, `specialty` matching a specialty name or code, `studyYear` 1 to 6, `status` defaulting to `ACTIVE`). Errors SHALL be 400 with the Java messages for a bad role, an unknown permission, a `student` on another role and a bad status, and 409 for a duplicate email or faculty number. A failed creation MUST leave no partial user. Created users MUST be able to sign in at once, with no forced password change.

#### Scenario: Default permissions
- **WHEN** a team creates a teacher without `permissions`
- **THEN** the user holds the `TEACHER` defaults

#### Scenario: Empty permissions
- **WHEN** a team creates a user with `"permissions": []`
- **THEN** the user holds no permissions

#### Scenario: Duplicate faculty number
- **WHEN** a team creates a student whose faculty number exists
- **THEN** the answer is 409 and no user is created

#### Scenario: Immediate sign-in
- **WHEN** a team creates a user and logs in through a module
- **THEN** the login completes without a password-change page

### Requirement: Shared data warning
The staging page SHALL state that all staging data is shared between teams and is wiped when the free database is replaced.

#### Scenario: Warning visible
- **WHEN** a team opens `/staging`
- **THEN** the page shows the shared-data warning
