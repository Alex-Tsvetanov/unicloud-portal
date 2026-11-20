## ADDED Requirements

### Requirement: Academic record
Every student SHALL have exactly one academic record: faculty number (at most 20 characters), specialty, study year (1 to 6) and student status (`ACTIVE`, `INTERRUPTED` or `GRADUATED`). A new record MUST default to `ACTIVE`. Only users with role `STUDENT` MAY have one.

#### Scenario: Record on another role
- **WHEN** a creator tries to attach an academic record to a teacher
- **THEN** the portal refuses with a Bulgarian message

#### Scenario: Study year out of range
- **WHEN** an inspector submits study year 7
- **THEN** the form rejects it

### Requirement: Faculty through specialty
A student's faculty SHALL be the faculty of the student's specialty. Teachers, deans and inspectors SHALL each belong to one faculty directly. Admins belong to none.

#### Scenario: Student faculty
- **WHEN** a student's specialty belongs to FCST
- **THEN** the student's profile and claims show faculty FCST

### Requirement: Own profile page
A signed-in user SHALL see their own profile at `/app/profile`: names, email, role in Bulgarian, faculty, and for a student the academic record with the status in Bulgarian (действащ, прекъснал, дипломиран).

#### Scenario: Student opens profile
- **WHEN** a student opens `/app/profile`
- **THEN** the page shows faculty number, specialty, study year, status and faculty

#### Scenario: Teacher opens profile
- **WHEN** a teacher opens `/app/profile`
- **THEN** the page shows names, email, role and faculty, and no academic record

### Requirement: Reading other students' profiles
The portal SHALL show another student's profile only to a user who holds a permission that covers that student: `STUDENT_PROFILE_READ_ANY` (every student), `STUDENT_PROFILE_READ_FACULTY` (students of the reader's faculty) or `STUDENT_PROFILE_READ_ENROLLED` (students enrolled in an offering the reader teaches). Anyone else MUST get 404, so the portal does not reveal that the student exists.

#### Scenario: Dean reads own faculty
- **WHEN** a dean of FCST opens the profile of a student whose specialty is in FCST
- **THEN** the profile is shown

#### Scenario: Dean reads another faculty
- **WHEN** a dean of FCST opens the profile of a student in another faculty
- **THEN** the portal answers 404

#### Scenario: Teacher reads an enrolled student
- **WHEN** a teacher opens the profile of a student enrolled in one of the teacher's offerings
- **THEN** the profile is shown

### Requirement: Changing student status
A user with `STUDENT_CREATE` SHALL be able to change a student's status. The change MUST take effect for enrollment checks and for tokens issued afterwards.

#### Scenario: Interrupt a student
- **WHEN** an inspector sets a student's status to `INTERRUPTED`
- **THEN** the student's profile shows "прекъснал"
- **AND** the student's next enrollment request is refused
