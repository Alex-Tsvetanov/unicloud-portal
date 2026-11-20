## ADDED Requirements

### Requirement: Client credentials grant
`POST /oauth2/token` SHALL accept `grant_type=client_credentials` with HTTP Basic client authentication and an optional `scope`. It SHALL issue an access token with scope `directory` only to modules granted `directory` at registration. The token has `sub` equal to the client id, lives 30 minutes, and comes with no ID token and no refresh token.

#### Scenario: Granted module
- **WHEN** a module with the `directory` grant asks for `scope=directory`
- **THEN** the answer is 200 with an access token whose `scope` is `directory`

#### Scenario: Module without the grant
- **WHEN** a module without the `directory` grant asks for `scope=directory`
- **THEN** the answer is 400 `invalid_scope`

#### Scenario: User scope requested
- **WHEN** a module asks for `scope=openid` with `client_credentials`
- **THEN** the answer is 400 `invalid_scope`

### Requirement: Directory access rule
Every route under `/api/v1/directory` SHALL require a Bearer client-credentials access token with scope `directory`. Any other token, an expired token, or no token MUST get 401. All routes MUST be read-only.

#### Scenario: User access token
- **WHEN** a module sends an access token from an authorization-code login
- **THEN** the directory answers 401

#### Scenario: Write attempt
- **WHEN** a module sends `POST /api/v1/directory/users/4`
- **THEN** the answer is 405

### Requirement: User lookup
`GET /api/v1/directory/users/{sub}` SHALL return `{sub, role, given_name, family_name, name, email, faculty, student}` using the same names and types as the SSO claims, with `faculty` and `student` omitted when the user has none. An unknown or disabled user SHALL give 404.

#### Scenario: Student lookup
- **WHEN** a module looks up a student
- **THEN** the answer has `role` `"STUDENT"`, `faculty` and `student.faculty_number`

### Requirement: A student's enrollments
`GET /api/v1/directory/users/{sub}/enrollments` SHALL return the student's `ENROLLED` offerings as `[{id, code, title, semester}]`. A user who is not a student SHALL give 404.

#### Scenario: Enrolled courses
- **WHEN** a student is enrolled in `MATH1` for `2026-27-W`
- **THEN** the list contains `{"code": "MATH1", "semester": "2026-27-W"}` with the offering id and title

### Requirement: A teacher's offerings
`GET /api/v1/directory/users/{sub}/offerings` SHALL return the offerings the teacher teaches as `[{id, code, title, semester}]`. A user who is not a teacher SHALL give 404.

#### Scenario: Teacher courses
- **WHEN** a module asks for a teacher's offerings
- **THEN** every item names a course code and a semester

### Requirement: Students of an offering
`GET /api/v1/directory/offerings/{id}/students` SHALL return the `ENROLLED` students as `[{sub, name, faculty_number}]`. An unknown offering SHALL give 404.

#### Scenario: Roster
- **WHEN** an offering has 3 enrolled students and 1 cancelled one
- **THEN** the list has 3 items

### Requirement: A faculty's offerings and students
`GET /api/v1/directory/faculties/{code}/offerings` SHALL return every offering of the faculty's courses as `[{id, code, title, semester, teacher_sub}]`. `GET /api/v1/directory/faculties/{code}/students` SHALL return the faculty's students as `[{sub, name, faculty_number, specialty, study_year, status}]`. Both MAY take `?semester=` (offerings) and SHALL return 404 for an unknown faculty code. Lists longer than 200 items SHALL be paged with `?page=` and a `next` link.

#### Scenario: Dean dashboard data
- **WHEN** Team 4 asks for `/api/v1/directory/faculties/FCST/offerings?semester=2026-27-W`
- **THEN** it gets every FCST offering of that semester

#### Scenario: Unknown faculty
- **WHEN** a module asks for faculty `XYZ`
- **THEN** the answer is 404

### Requirement: Directory documentation
`docs/sso.md` SHALL document the directory API, how to get the `directory` grant on staging and in production, and the shared course identifier (the course code).

#### Scenario: Module team reads the guide
- **WHEN** a module team opens `docs/sso.md`
- **THEN** it finds the client-credentials request, every directory route and its response shape
