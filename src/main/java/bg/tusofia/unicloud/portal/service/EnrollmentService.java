package bg.tusofia.unicloud.portal.service;

import java.util.List;

import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import bg.tusofia.unicloud.portal.dto.EnrollmentRequest;
import bg.tusofia.unicloud.portal.dto.EnrollmentResponse;
import bg.tusofia.unicloud.portal.entity.CourseOffering;
import bg.tusofia.unicloud.portal.entity.Enrollment;
import bg.tusofia.unicloud.portal.entity.Student;
import bg.tusofia.unicloud.portal.exception.EnrollmentException;
import bg.tusofia.unicloud.portal.exception.EnrollmentException.ErrorCode;
import bg.tusofia.unicloud.portal.exception.ResourceNotFoundException;
import bg.tusofia.unicloud.portal.repository.CourseOfferingRepository;
import bg.tusofia.unicloud.portal.repository.EnrollmentRepository;
import bg.tusofia.unicloud.portal.repository.StudentRepository;

@Service
public class EnrollmentService {

    private final EnrollmentRepository enrollmentRepository;
    private final StudentRepository studentRepository;
    private final CourseOfferingRepository offeringRepository;
    private final JdbcTemplate jdbc;

    public EnrollmentService(EnrollmentRepository enrollmentRepository,
                             StudentRepository studentRepository,
                             CourseOfferingRepository offeringRepository,
                             JdbcTemplate jdbc) {
        this.enrollmentRepository = enrollmentRepository;
        this.studentRepository = studentRepository;
        this.offeringRepository = offeringRepository;
        this.jdbc = jdbc;
    }

    @Transactional(readOnly = true)
    public List<EnrollmentResponse> findByStudentId(Long studentId) {
        // Verify student exists
        if (!studentRepository.existsById(studentId)) {
            throw new ResourceNotFoundException("Student", "id", studentId);
        }
        return enrollmentRepository.findByStudentUserId(studentId).stream()
                .map(EnrollmentResponse::from)
                .toList();
    }

    @Transactional(readOnly = true)
    public EnrollmentResponse findById(Long id) {
        Enrollment enrollment = enrollmentRepository.findById(id)
                .orElseThrow(() -> new ResourceNotFoundException("Enrollment", "id", id));
        return EnrollmentResponse.from(enrollment);
    }

    /**
     * Enroll a student in a course offering.
     * Uses row-level locking to prevent overselling seats.
     */
    @Transactional
    public EnrollmentResponse enroll(EnrollmentRequest request) {
        Long studentId = request.studentId();
        Long offeringId = request.offeringId();

        // 1. Verify student exists and is ACTIVE
        Student student = studentRepository.findById(studentId)
                .orElseThrow(() -> new ResourceNotFoundException("Student", "id", studentId));

        if (student.getStatus() != Student.Status.ACTIVE) {
            throw new EnrollmentException(ErrorCode.STUDENT_NOT_ACTIVE);
        }

        // 2. Lock the offering row and check if open
        Integer capacity = jdbc.query(
                "SELECT capacity FROM course_offerings WHERE id = ? AND is_open = TRUE FOR UPDATE",
                rs -> rs.next() ? rs.getInt(1) : null,
                offeringId
        );

        if (capacity == null) {
            // Check if offering exists but is closed, or doesn't exist at all
            boolean exists = offeringRepository.existsById(offeringId);
            if (!exists) {
                throw new ResourceNotFoundException("CourseOffering", "id", offeringId);
            }
            throw new EnrollmentException(ErrorCode.COURSE_CLOSED);
        }

        // 3. Count enrolled students (with shared lock to see latest committed data)
        Integer enrolled = jdbc.queryForObject(
                "SELECT COUNT(*) FROM enrollments WHERE offering_id = ? AND status = 'ENROLLED' FOR SHARE",
                Integer.class,
                offeringId
        );

        if (enrolled >= capacity) {
            throw new EnrollmentException(ErrorCode.COURSE_FULL);
        }

        // 4. Check for existing enrollment
        var existingEnrollment = enrollmentRepository.findByStudentUserIdAndOfferingId(studentId, offeringId);

        if (existingEnrollment.isPresent()) {
            Enrollment existing = existingEnrollment.get();
            if (existing.getStatus() == Enrollment.Status.ENROLLED) {
                throw new EnrollmentException(ErrorCode.ALREADY_ENROLLED);
            }
            // Re-enroll: change CANCELLED back to ENROLLED
            existing.setStatus(Enrollment.Status.ENROLLED);
            Enrollment saved = enrollmentRepository.save(existing);
            return EnrollmentResponse.from(saved);
        }

        // 5. Create new enrollment
        CourseOffering offering = offeringRepository.findById(offeringId)
                .orElseThrow(() -> new ResourceNotFoundException("CourseOffering", "id", offeringId));

        Enrollment enrollment = new Enrollment(student, offering);
        try {
            Enrollment saved = enrollmentRepository.save(enrollment);
            return EnrollmentResponse.from(saved);
        } catch (DataIntegrityViolationException e) {
            // Concurrent request already enrolled this student
            throw new EnrollmentException(ErrorCode.ALREADY_ENROLLED);
        }
    }

    /**
     * Cancel an enrollment.
     * Uses row-level locking for consistency.
     */
    @Transactional
    public EnrollmentResponse cancel(Long enrollmentId) {
        Enrollment enrollment = enrollmentRepository.findById(enrollmentId)
                .orElseThrow(() -> new EnrollmentException(ErrorCode.ENROLLMENT_NOT_FOUND));

        if (enrollment.getStatus() == Enrollment.Status.CANCELLED) {
            throw new EnrollmentException(ErrorCode.ALREADY_CANCELLED);
        }

        // Lock the offering row first for consistency
        jdbc.query(
                "SELECT id FROM course_offerings WHERE id = ? FOR UPDATE",
                rs -> null,
                enrollment.getOffering().getId()
        );

        enrollment.setStatus(Enrollment.Status.CANCELLED);
        Enrollment saved = enrollmentRepository.save(enrollment);
        return EnrollmentResponse.from(saved);
    }
}
