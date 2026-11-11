-- UniCloud portal, initial schema (MySQL 8.4, InnoDB, utf8mb4).
-- In third normal form (and BCNF): every column depends on its table's key, the whole key
-- and nothing but the key.
-- Flyway runs this once on application start. Never edit it after it has run:
-- put later changes in V2__..., V3__... files.

-- Everyone who can log in.
CREATE TABLE users (
    id             BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    email          VARCHAR(254)    NOT NULL,
    password_hash  VARCHAR(255)    NOT NULL,   -- BCrypt or Argon2 hash, never the password itself
    first_name     VARCHAR(100)    NOT NULL,
    last_name      VARCHAR(100)    NOT NULL,
    role           VARCHAR(16)     NOT NULL,
    enabled        BOOLEAN         NOT NULL DEFAULT TRUE,
    created_at     DATETIME(3)     NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at     DATETIME(3)     NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uq_users_email (email),
    CONSTRAINT chk_users_role CHECK (role IN ('STUDENT', 'TEACHER', 'INSPECTOR', 'ADMIN'))
);

-- Academic data, one row per student (one-to-one with users).
CREATE TABLE students (
    user_id         BIGINT UNSIGNED  NOT NULL,
    faculty_number  VARCHAR(20)      NOT NULL,
    specialty       VARCHAR(200)     NOT NULL,
    study_year      TINYINT UNSIGNED NOT NULL,
    status          VARCHAR(16)      NOT NULL DEFAULT 'ACTIVE',
    PRIMARY KEY (user_id),
    UNIQUE KEY uq_students_faculty_number (faculty_number),
    CONSTRAINT fk_students_user FOREIGN KEY (user_id) REFERENCES users (id),
    CONSTRAINT chk_students_study_year CHECK (study_year BETWEEN 1 AND 6),
    CONSTRAINT chk_students_status CHECK (status IN ('ACTIVE', 'INTERRUPTED', 'GRADUATED'))
);

-- The catalogue: facts about a course that don't change between semesters.
CREATE TABLE courses (
    id            INT UNSIGNED     NOT NULL AUTO_INCREMENT,
    code          VARCHAR(16)      NOT NULL,
    title         VARCHAR(200)     NOT NULL,
    ects_credits  TINYINT UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_courses_code (code)
);

-- A course running in one semester, with its teacher and seat limit.
CREATE TABLE course_offerings (
    id           BIGINT UNSIGNED   NOT NULL AUTO_INCREMENT,
    course_id    INT UNSIGNED      NOT NULL,
    semester     VARCHAR(16)       NOT NULL,   -- e.g. 2026-27-W
    teacher_id   BIGINT UNSIGNED   NOT NULL,
    capacity     SMALLINT UNSIGNED NOT NULL,
    is_open      BOOLEAN           NOT NULL DEFAULT TRUE,
    PRIMARY KEY (id),
    UNIQUE KEY uq_course_offerings_course_semester (course_id, semester),
    KEY ix_course_offerings_semester (semester),
    KEY ix_course_offerings_teacher (teacher_id),
    CONSTRAINT fk_course_offerings_course FOREIGN KEY (course_id) REFERENCES courses (id),
    CONSTRAINT fk_course_offerings_teacher FOREIGN KEY (teacher_id) REFERENCES users (id),
    CONSTRAINT chk_course_offerings_capacity CHECK (capacity > 0)
);

-- Who is enrolled in what. It references students, so only students can enroll.
CREATE TABLE enrollments (
    id           BIGINT UNSIGNED NOT NULL AUTO_INCREMENT,
    student_id   BIGINT UNSIGNED NOT NULL,
    offering_id  BIGINT UNSIGNED NOT NULL,
    status       VARCHAR(16)     NOT NULL DEFAULT 'ENROLLED',
    created_at   DATETIME(3)     NOT NULL DEFAULT CURRENT_TIMESTAMP(3),
    updated_at   DATETIME(3)     NOT NULL DEFAULT CURRENT_TIMESTAMP(3) ON UPDATE CURRENT_TIMESTAMP(3),
    PRIMARY KEY (id),
    UNIQUE KEY uq_enrollments_student_offering (student_id, offering_id),   -- stops double clicks
    KEY ix_enrollments_offering_status (offering_id, status),               -- counts taken seats fast
    CONSTRAINT fk_enrollments_student FOREIGN KEY (student_id) REFERENCES students (user_id),
    CONSTRAINT fk_enrollments_offering FOREIGN KEY (offering_id) REFERENCES course_offerings (id),
    CONSTRAINT chk_enrollments_status CHECK (status IN ('ENROLLED', 'CANCELLED'))
);

-- Taken seats are counted, not stored, so no counter can drift out of sync.
-- Taking a seat, in one transaction (the app runs this; shown for reference):
--   SELECT capacity FROM course_offerings WHERE id = ? AND is_open FOR UPDATE;
--       no row = closed: roll back. The row lock makes other enrollments in this course wait their turn.
--   SELECT COUNT(*) FROM enrollments WHERE offering_id = ? AND status = 'ENROLLED' FOR SHARE;
--       count >= capacity = full: roll back. A locking read always sees the latest committed rows.
--   INSERT INTO enrollments (student_id, offering_id) VALUES (?, ?);
--       duplicate key = already enrolled: roll back.
-- Cancelling and re-enrolling take the same row lock first, then change the enrollment's status.
