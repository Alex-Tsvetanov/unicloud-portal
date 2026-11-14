-- Test data for development and demo purposes.
-- Run after V1__init_schema.sql has created the tables.

-- Teachers (users with TEACHER role)
INSERT INTO users (email, password_hash, first_name, last_name, role, enabled) VALUES
('ivan.petrov@tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000000', 'Иван', 'Петров', 'TEACHER', TRUE),
('maria.ivanova@tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000001', 'Мария', 'Иванова', 'TEACHER', TRUE),
('georgi.dimitrov@tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000002', 'Георги', 'Димитров', 'TEACHER', TRUE);

-- Students (users with STUDENT role + student records)
INSERT INTO users (email, password_hash, first_name, last_name, role, enabled) VALUES
('student1@student.tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000003', 'Александър', 'Стоянов', 'STUDENT', TRUE),
('student2@student.tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000004', 'Елена', 'Георгиева', 'STUDENT', TRUE),
('student3@student.tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000005', 'Николай', 'Тодоров', 'STUDENT', TRUE),
('student4@student.tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000006', 'Виктория', 'Николова', 'STUDENT', TRUE),
('student5@student.tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000007', 'Димитър', 'Петков', 'STUDENT', TRUE);

INSERT INTO students (user_id, faculty_number, specialty, study_year, status) VALUES
(4, '121219001', 'Компютърни науки', 2, 'ACTIVE'),
(5, '121219002', 'Компютърни науки', 2, 'ACTIVE'),
(6, '121219003', 'Софтуерно инженерство', 3, 'ACTIVE'),
(7, '121219004', 'Софтуерно инженерство', 3, 'INTERRUPTED'),
(8, '121218001', 'Компютърни науки', 4, 'ACTIVE');

-- Admin user
INSERT INTO users (email, password_hash, first_name, last_name, role, enabled) VALUES
('admin@tu-sofia.bg', '$2a$10$dummyhashfordevonly000000000000000000000000000000008', 'Admin', 'User', 'ADMIN', TRUE);

-- Courses
INSERT INTO courses (code, title, ects_credits) VALUES
('CS101', 'Въведение в програмирането', 6),
('CS201', 'Структури от данни и алгоритми', 6),
('CS301', 'Бази данни', 5),
('CS302', 'Уеб технологии', 5),
('CS401', 'Софтуерно инженерство', 6),
('CS402', 'Разпределени системи', 6),
('MATH101', 'Линейна алгебра', 5),
('MATH201', 'Дискретна математика', 5);

-- Course offerings for current semester (2026-27-W = Winter 2026-27)
INSERT INTO course_offerings (course_id, semester, teacher_id, capacity, is_open) VALUES
(1, '2026-27-W', 1, 30, TRUE),   -- CS101 by Ivan Petrov, 30 seats
(2, '2026-27-W', 1, 25, TRUE),   -- CS201 by Ivan Petrov, 25 seats
(3, '2026-27-W', 2, 20, TRUE),   -- CS301 by Maria Ivanova, 20 seats
(4, '2026-27-W', 2, 25, TRUE),   -- CS302 by Maria Ivanova, 25 seats
(5, '2026-27-W', 3, 15, TRUE),   -- CS401 by Georgi Dimitrov, 15 seats
(6, '2026-27-W', 3, 3, TRUE),    -- CS402 by Georgi Dimitrov, only 3 seats (for testing capacity)
(7, '2026-27-W', 1, 40, FALSE),  -- MATH101 closed for enrollment
(8, '2026-27-W', 2, 35, TRUE);   -- MATH201 open

-- Some initial enrollments
INSERT INTO enrollments (student_id, offering_id, status) VALUES
(4, 1, 'ENROLLED'),  -- Aleksandar in CS101
(4, 2, 'ENROLLED'),  -- Aleksandar in CS201
(5, 1, 'ENROLLED'),  -- Elena in CS101
(5, 3, 'ENROLLED'),  -- Elena in CS301
(6, 5, 'ENROLLED'),  -- Nikolay in CS401
(8, 6, 'ENROLLED');  -- Dimitar in CS402 (1 of 3 seats taken)
