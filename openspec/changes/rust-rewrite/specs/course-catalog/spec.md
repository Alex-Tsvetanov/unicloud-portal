## ADDED Requirements

### Requirement: Faculties and specialties
The portal SHALL keep faculties, each with a unique short code (for example `FCST`) and a Bulgarian name, and specialties that each belong to one faculty.

#### Scenario: Seed data
- **WHEN** the portal starts on an empty database
- **THEN** the migrations create at least one faculty with its specialties

### Requirement: Courses
A course SHALL have a unique course code (at most 16 characters, for example `MATH1`), a title and ECTS credits. The course code MUST be the course identifier shared with other UniCloud products. Courses MUST NOT be hard-deleted.

#### Scenario: Create a course
- **WHEN** a user with `COURSE_CREATE_MODIFY` creates course `MATH1`
- **THEN** the course appears in the catalogue

#### Scenario: Duplicate code
- **WHEN** a user creates a second course with code `MATH1`
- **THEN** the form shows an error and nothing is created

#### Scenario: No permission
- **WHEN** a student posts the new-course form
- **THEN** the portal answers 403

### Requirement: Course offerings
A course offering SHALL be one course in one semester (for example `2026-27-W`), with one teacher from the course's faculty, a capacity of at least 1, and an open or closed flag. It SHALL be identified by a numeric id and, for other products, by `{id, code, semester}`. Offerings MUST NOT be hard-deleted.

#### Scenario: Create an offering
- **WHEN** a user with `COURSE_OFFERING_CREATE_MODIFY` creates an offering of `MATH1` for `2026-27-W` with capacity 30 and a teacher
- **THEN** the offering is listed with 30 free places

#### Scenario: Capacity below enrolled count
- **WHEN** a user lowers an offering's capacity below the number of students already enrolled
- **THEN** the portal refuses the change

### Requirement: Catalogue page
A user with `COURSE_OFFERING_READ` SHALL see the catalogue of open offerings for the current semester at `/app/courses`, with course code, title, ECTS, teacher and free places. Filtering by faculty and searching by code or title SHALL update the list without a full page load.

#### Scenario: Filter by faculty
- **WHEN** a student picks FCST in the faculty filter
- **THEN** the list shows only FCST offerings
- **AND** the page URL reflects the filter so it can be shared

#### Scenario: Free places
- **WHEN** an offering has capacity 30 and 28 enrollments
- **THEN** the catalogue shows 2 free places

### Requirement: Staff management pages
Users with `COURSE_CREATE_MODIFY` or `COURSE_OFFERING_CREATE_MODIFY` SHALL manage courses and offerings from portal pages. These controls MUST NOT appear for users without the permission.

#### Scenario: Student view
- **WHEN** a student opens `/app/courses`
- **THEN** there are no create or edit controls on the page
