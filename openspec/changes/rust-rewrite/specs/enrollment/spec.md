## ADDED Requirements

### Requirement: Enrollment request takes a place at once
A student with `ENROLLMENT_REQUEST_CREATE` and status `ACTIVE` SHALL enroll in an open offering that has a free place. The place MUST be taken in the same transaction as the request. The enrollment status is `ENROLLED`.

#### Scenario: Free place
- **WHEN** an active student presses "Запиши се" on an open offering with 1 free place
- **THEN** the enrollment is `ENROLLED` and the offering shows 0 free places
- **AND** the button is replaced through an htmx fragment, without a full page load

#### Scenario: Full offering
- **WHEN** a student presses "Запиши се" on an offering with 0 free places
- **THEN** the portal answers with "Няма свободни места" and creates nothing

#### Scenario: Closed offering
- **WHEN** a student tries to enroll in a closed offering
- **THEN** the portal refuses

#### Scenario: Inactive student
- **WHEN** a student with status `INTERRUPTED` or `GRADUATED` tries to enroll
- **THEN** the portal refuses with a Bulgarian message

#### Scenario: Already enrolled
- **WHEN** a student who is `ENROLLED` in an offering enrolls again
- **THEN** the portal answers that the student is already enrolled and takes no second place

### Requirement: No overselling under load
The portal MUST never let the number of `ENROLLED` students in an offering exceed its capacity, even when many requests for the last places arrive at the same moment on different replicas.

#### Scenario: Race for the last place
- **WHEN** 50 different students request the last free place at the same moment
- **THEN** exactly 1 enrollment succeeds and 49 get "Няма свободни места"

### Requirement: Cancel by the student
A student SHALL cancel their own enrollment. The status becomes `CANCELLED` and the place is freed. A later request MAY enroll the student again if a place is free.

#### Scenario: Cancel and re-enroll
- **WHEN** a student cancels and then enrolls again while a place is free
- **THEN** the same enrollment record returns to `ENROLLED`

### Requirement: Rejection by the teacher
The teacher of an offering, holding `ENROLLMENT_REQUEST_ACCEPT_REJECT`, SHALL reject an enrollment in that offering. The status becomes `REJECTED` and the place is freed. A rejected student MUST NOT enroll in the same offering again.

#### Scenario: Teacher rejects
- **WHEN** the offering's teacher rejects a student's enrollment
- **THEN** the enrollment is `REJECTED` and the offering has one more free place

#### Scenario: Another teacher
- **WHEN** a teacher who does not teach the offering tries to reject an enrollment in it
- **THEN** the portal answers 403

#### Scenario: Rejected student retries
- **WHEN** a rejected student tries to enroll in the same offering again
- **THEN** the portal refuses

### Requirement: Enrollment pages per role
A student SHALL see their enrollments on `/enrollments`. A teacher SHALL see, per offering they teach, the list of enrolled students with a reject control.

#### Scenario: Teacher roster
- **WHEN** a teacher opens one of their offerings
- **THEN** the page lists enrolled students with faculty number and name, and a "Откажи" control per student
