# UniCloud Portal

The university's single entry point and its single source of truth for who is who: people, their roles, faculties, courses and enrollments. Every other UniCloud product signs its users in through the portal.

The UI is in Bulgarian. The Bulgarian word for each term is in brackets.

## Ecosystem

**Portal** (портал):
This product, built by Team 1. It holds identity, academic profiles and enrollment, and it signs people in for every module.
_Avoid_: identity service, SSO server, auth service

**Module** (модул):
Another UniCloud product that signs its users in through the portal, for example Team 2's courseware or Team 3's autograder.
_Avoid_: client, app, relying party, service

**SSO session** (единен вход):
The proof, kept in one browser, that a person signed in at the portal. Modules reuse it so the person does not type the password again.
_Avoid_: login, portal token

## People

**User** (потребител):
A person with an account at the portal. Every user has exactly one role.
_Avoid_: account, member, principal

**Role** (роля):
What kind of user someone is: Student, Teacher, Dean, Inspector or Admin.
_Avoid_: user type, group

**Student** (студент):
A user who studies at the university and has an academic record.

**Teacher** (преподавател):
A user who teaches course offerings.
_Avoid_: Professor, lecturer, instructor

**Dean** (декан):
A user who leads one faculty and oversees its students and courses. A faculty has at most one dean.

**Inspector** (инспектор от учебен отдел):
A staff member of the academic office who creates students and manages courses and offerings.
_Avoid_: clerk, registrar, academic admin

**Admin** (администратор):
A user who creates staff accounts and registers modules. Admins have no academic powers by default.
_Avoid_: SysAdmin, superuser

**Permission** (право):
One named thing a user may do. A role gives a default set of permissions, and a user's set can differ from that default.

## Academic structure

**Faculty** (факултет):
An academic unit of the university, such as FCST. Specialties, teachers, deans and inspectors each belong to one faculty. A student belongs to the faculty of their specialty.
_Avoid_: department, school

**Specialty** (специалност):
A degree programme within one faculty, for example Computer Science.
_Avoid_: major, programme, degree

**Academic record** (академичен профил):
A student's faculty number, specialty, study year and student status.
_Avoid_: student profile, student data

**Faculty number** (факултетен номер):
The number that identifies one student across the university.
_Avoid_: student ID, student number

**Student status** (статус):
Whether a student is Active (действащ), Interrupted (прекъснал) or Graduated (дипломиран).

## Courses

**Course** (дисциплина):
A subject in the catalogue, with a code, a title and ECTS credits. It does not belong to a semester.
_Avoid_: subject, class

**Course offering** (курс):
One course taught in one semester by one teacher, with a limited number of places.
_Avoid_: section, class, course instance

**Course code** (код на дисциплина):
The code that names one course across every UniCloud product, for example MATH1. Other teams use it as their course identifier.
_Avoid_: CourseID, subject code

**Enrollment** (записване):
A student's place in one course offering. A student's request takes a free place at once. The offering's teacher may later reject it, which frees the place.
_Avoid_: enrolment, registration, sign-up, enrollment request
