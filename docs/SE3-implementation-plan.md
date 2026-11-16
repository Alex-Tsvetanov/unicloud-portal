# SE3 Implementation Plan — Academic Services & Enrollment

**Engineer:** Martin (Software Engineer 3)  
**Created:** 2026-11-14  
**Status:** In Progress

---

## Scope Summary

Build the Course Catalog and Enrollment APIs for UniCloud Portal.

### Constraints
- No authentication for now — role-based access added later when SE1 delivers identity service
- No hard deletes — courses and offerings must remain for historical records
- Teacher assigned via `teacher_id` in offering creation

### Out of Scope
- Authentication/Authorization (SE1 responsibility)
- User/Student CRUD (SE1 responsibility)
- Delete operations for courses/offerings

---

## API Contract

### Course Management

| Method | Endpoint | Request Body | Response | Description |
|--------|----------|--------------|----------|-------------|
| `POST` | `/api/v1/courses` | `{code, title, ectsCredits}` | `201 Created` + Course | Create course |
| `GET` | `/api/v1/courses` | — | `200 OK` + Course[] | List all courses |
| `GET` | `/api/v1/courses/{id}` | — | `200 OK` + Course | Get course by ID |
| `PUT` | `/api/v1/courses/{id}` | `{code, title, ectsCredits}` | `200 OK` + Course | Update course |

### Course Offerings

| Method | Endpoint | Request Body | Response | Description |
|--------|----------|--------------|----------|-------------|
| `POST` | `/api/v1/offerings` | `{courseId, semester, teacherId, capacity, isOpen}` | `201 Created` + Offering | Create offering |
| `GET` | `/api/v1/offerings` | Query: `?semester=&courseId=&open=` | `200 OK` + Offering[] | List offerings (with filters) |
| `GET` | `/api/v1/offerings/{id}` | — | `200 OK` + OfferingDetail | Get offering with available seats |
| `PUT` | `/api/v1/offerings/{id}` | `{capacity, isOpen}` | `200 OK` + Offering | Update offering |

### Enrollment

| Method | Endpoint | Request Body | Response | Description |
|--------|----------|--------------|----------|-------------|
| `POST` | `/api/v1/enrollments` | `{studentId, offeringId}` | `201 Created` + Enrollment | Enroll student |
| `GET` | `/api/v1/enrollments/{id}` | — | `200 OK` + Enrollment | Get enrollment by ID |
| `PATCH` | `/api/v1/enrollments/{id}/cancel` | — | `200 OK` + Enrollment | Cancel enrollment |
| `GET` | `/api/v1/students/{id}/enrollments` | — | `200 OK` + Enrollment[] | Get student's enrollments |

---

## Implementation Checklist

### Phase 1: Foundation (JPA Entities & Repositories) ✅

- [x] **1.1** Create `User` entity
- [x] **1.2** Create `Student` entity
- [x] **1.3** Create `Course` entity
- [x] **1.4** Create `CourseOffering` entity
- [x] **1.5** Create `Enrollment` entity
- [x] **1.6** Create `UserRepository`
- [x] **1.7** Create `StudentRepository`
- [x] **1.8** Create `CourseRepository`
- [x] **1.9** Create `CourseOfferingRepository`
- [x] **1.10** Create `EnrollmentRepository`
- [x] **1.11** Verify app compiles successfully

### Phase 2: DTOs & Exception Handling ✅

- [x] **2.1** Create `CourseRequest` DTO
- [x] **2.2** Create `CourseResponse` DTO
- [x] **2.3** Create `OfferingRequest` DTO
- [x] **2.4** Create `OfferingResponse` DTO (with available seats)
- [x] **2.5** Create `EnrollmentRequest` DTO
- [x] **2.6** Create `EnrollmentResponse` DTO
- [x] **2.7** Create `ApiError` response DTO
- [x] **2.8** Create `GlobalExceptionHandler` for consistent error responses

### Phase 3: Course Catalog API ✅

- [x] **3.1** Create `CourseService`
- [x] **3.2** Create `CourseController`
- [x] **3.3** Implement `POST /api/v1/courses`
- [x] **3.4** Implement `GET /api/v1/courses`
- [x] **3.5** Implement `GET /api/v1/courses/{id}`
- [x] **3.6** Implement `PUT /api/v1/courses/{id}`
- [x] **3.7** Add input validation (code format, title length, ECTS range)

### Phase 4: Course Offerings API ✅

- [x] **4.1** Create `CourseOfferingService`
- [x] **4.2** Create `CourseOfferingController`
- [x] **4.3** Implement `POST /api/v1/offerings`
- [x] **4.4** Implement `GET /api/v1/offerings` with filters
- [x] **4.5** Implement `GET /api/v1/offerings/{id}` with available seats calculation
- [x] **4.6** Implement `PUT /api/v1/offerings/{id}`
- [x] **4.7** Add input validation (capacity > 0, semester format)

### Phase 5: Enrollment API (Concurrency-Safe) ✅

- [x] **5.1** Create `EnrollmentService` with `JdbcTemplate`
- [x] **5.2** Implement `enroll()` with row-level locking (`SELECT ... FOR UPDATE`)
- [x] **5.3** Implement `cancel()` with row-level locking
- [x] **5.4** Create `EnrollmentController`
- [x] **5.5** Implement `POST /api/v1/enrollments`
- [x] **5.6** Implement `PATCH /api/v1/enrollments/{id}/cancel`
- [x] **5.7** Implement `GET /api/v1/students/{id}/enrollments`
- [x] **5.8** Handle business rules:
  - [x] Only ACTIVE students can enroll
  - [x] Offering must be open (`is_open = true`)
  - [x] Capacity not exceeded
  - [x] No duplicate enrollments
  - [x] Handle re-enrollment after cancellation

### Phase 6: Test Data ✅

- [x] **6.1** Create `V2__test_data.sql` migration with:
  - [x] Sample users (teachers)
  - [x] Sample students
  - [x] Sample courses
  - [x] Sample offerings for current semester

### Phase 7: Testing (Deferred)

- [ ] **7.1** Unit tests for `CourseService`
- [ ] **7.2** Unit tests for `CourseOfferingService`
- [ ] **7.3** Unit tests for `EnrollmentService`
- [ ] **7.4** Integration tests for Course API
- [ ] **7.5** Integration tests for Offering API
- [ ] **7.6** Integration tests for Enrollment API
- [ ] **7.7** **Concurrency test**: simulate 08:00 enrollment rush (multiple threads enrolling for last seat)
- [ ] **7.8** Run `./mvnw verify` — all tests pass

### Phase 8: Documentation (Deferred)

- [ ] **8.1** Update README with new API endpoints
- [ ] **8.2** Add API documentation (endpoint descriptions, request/response examples)
- [ ] **8.3** Document enrollment business rules

### Phase 9: Refactoring & Role-Based Access Control (Future — after SE1 delivers identity)

- [ ] **9.1** Integrate with SE1 authentication service
- [ ] **9.2** Add Spring Security configuration
- [ ] **9.3** Add role-based access control:
  - [ ] `POST /api/v1/courses` — INSPECTOR, ADMIN only
  - [ ] `PUT /api/v1/courses/{id}` — INSPECTOR, ADMIN only
  - [ ] `POST /api/v1/offerings` — INSPECTOR, ADMIN only
  - [ ] `PUT /api/v1/offerings/{id}` — INSPECTOR, ADMIN only
  - [ ] `POST /api/v1/enrollments` — STUDENT (own) only
  - [ ] `PATCH /api/v1/enrollments/{id}/cancel` — STUDENT (own), ADMIN
  - [ ] `GET /api/v1/students/{id}/enrollments` — STUDENT (own), TEACHER, INSPECTOR, ADMIN
- [ ] **9.4** Code refactoring and cleanup
- [ ] **9.5** Performance optimizations if needed

---

## Files in This PR

### New Files (24 files)

**Entities (5):**
- `src/main/java/bg/tusofia/unicloud/portal/entity/User.java`
- `src/main/java/bg/tusofia/unicloud/portal/entity/Student.java`
- `src/main/java/bg/tusofia/unicloud/portal/entity/Course.java`
- `src/main/java/bg/tusofia/unicloud/portal/entity/CourseOffering.java`
- `src/main/java/bg/tusofia/unicloud/portal/entity/Enrollment.java`

**Repositories (5):**
- `src/main/java/bg/tusofia/unicloud/portal/repository/UserRepository.java`
- `src/main/java/bg/tusofia/unicloud/portal/repository/StudentRepository.java`
- `src/main/java/bg/tusofia/unicloud/portal/repository/CourseRepository.java`
- `src/main/java/bg/tusofia/unicloud/portal/repository/CourseOfferingRepository.java`
- `src/main/java/bg/tusofia/unicloud/portal/repository/EnrollmentRepository.java`

**DTOs (7):**
- `src/main/java/bg/tusofia/unicloud/portal/dto/CourseRequest.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/CourseResponse.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/OfferingRequest.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/OfferingResponse.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/EnrollmentRequest.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/EnrollmentResponse.java`
- `src/main/java/bg/tusofia/unicloud/portal/dto/ApiError.java`

**Services (3):**
- `src/main/java/bg/tusofia/unicloud/portal/service/CourseService.java`
- `src/main/java/bg/tusofia/unicloud/portal/service/CourseOfferingService.java`
- `src/main/java/bg/tusofia/unicloud/portal/service/EnrollmentService.java`

**Controllers (3):**
- `src/main/java/bg/tusofia/unicloud/portal/controller/CourseController.java`
- `src/main/java/bg/tusofia/unicloud/portal/controller/CourseOfferingController.java`
- `src/main/java/bg/tusofia/unicloud/portal/controller/EnrollmentController.java`

**Exceptions (3):**
- `src/main/java/bg/tusofia/unicloud/portal/exception/ResourceNotFoundException.java`
- `src/main/java/bg/tusofia/unicloud/portal/exception/DuplicateResourceException.java`
- `src/main/java/bg/tusofia/unicloud/portal/exception/EnrollmentException.java`
- `src/main/java/bg/tusofia/unicloud/portal/exception/GlobalExceptionHandler.java`

**Database Migration (1):**
- `src/main/resources/db/migration/V2__test_data.sql`

**Documentation (1):**
- `docs/SE3-implementation-plan.md`

### Modified Files (1)

- `pom.xml` — Added `spring-boot-starter-validation` dependency

---

## Enrollment Logic (Critical Path)

From README — must follow this pattern exactly:

```java
@Transactional
public void enroll(long studentId, long offeringId) {
    // 1. Verify student is ACTIVE
    // 2. Lock offering row: SELECT capacity FROM course_offerings WHERE id = ? AND is_open FOR UPDATE
    //    - No row or is_open=false → throw "Course is closed"
    // 3. Count enrolled: SELECT COUNT(*) FROM enrollments WHERE offering_id = ? AND status = 'ENROLLED' FOR SHARE
    //    - count >= capacity → throw "Course is full"
    // 4. Check existing enrollment for this student+offering
    //    - If exists with CANCELLED → UPDATE to ENROLLED
    //    - If exists with ENROLLED → throw "Already enrolled"
    //    - If not exists → INSERT
}
```

---

## Progress Log

| Date | Phase | Items Completed | Notes |
|------|-------|-----------------|-------|
| 2026-11-14 | 1-6 | Phases 1-6 complete | Ready for PR |

---

## Definition of Done (This PR)

- [x] All Phase 1-6 checklist items completed
- [x] `mvn compile` passes
- [ ] Code reviewed and merged to `main`
