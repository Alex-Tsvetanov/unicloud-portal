-- Login and registration (user.domain.User / UserJPAEntity) keep each user's permissions as a
-- bitmask in users.permissions_mask, but no migration ever added the column, so Hibernate's
-- schema validation (spring.jpa.hibernate.ddl-auto=validate) refused to start the app.
-- Bit positions are those of bg.tusofia.unicloud.portal.user.domain.Permission.
ALTER TABLE users
    ADD COLUMN permissions_mask BIGINT UNSIGNED NOT NULL DEFAULT 0 AFTER role;

-- Rows that already exist (the V2 test data included) get their role's default permissions,
-- the same mask bg.tusofia.unicloud.portal.user.domain.Role gives a newly registered user.
UPDATE users SET permissions_mask = CASE role
    WHEN 'ADMIN'     THEN (1 << 0)                                              -- STAFF_CREATE
    WHEN 'INSPECTOR' THEN (1 << 1) | (1 << 3) | (1 << 5) | (1 << 6) | (1 << 7)  -- STUDENT_CREATE, STUDENT_PROFILE_READ_ANY, COURSE_CREATE_MODIFY, COURSE_OFFERING_READ, COURSE_OFFERING_CREATE_MODIFY
    WHEN 'TEACHER'   THEN (1 << 4) | (1 << 6) | (1 << 9)                        -- STUDENT_PROFILE_READ_ENROLLED, COURSE_OFFERING_READ, ENROLLMENT_REQUEST_ACCEPT_REJECT
    WHEN 'STUDENT'   THEN (1 << 2) | (1 << 6) | (1 << 8)                        -- STUDENT_PROFILE_READ_OWN, COURSE_OFFERING_READ, ENROLLMENT_REQUEST_CREATE
    ELSE 0
END;
