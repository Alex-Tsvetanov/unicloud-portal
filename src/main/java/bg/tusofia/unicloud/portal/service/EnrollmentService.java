package bg.tusofia.unicloud.portal.service;

import java.util.List;

import org.springframework.dao.DataIntegrityViolationException;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Isolation;
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

    public EnrollmentService(EnrollmentRepository enrollmentRepository,
                             StudentRepository studentRepository,
                             CourseOfferingRepository offeringRepository) {
        this.enrollmentRepository = enrollmentRepository;
        this.studentRepository = studentRepository;
        this.offeringRepository = offeringRepository;
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
     * Enroll a student in a course offering without overselling seats.
     *
     * <p>Two things keep the seat count right on every database, with the same code:
     * <ul>
     *   <li>The offering's row is locked first, so enrollments in the same offering run one
     *       after another.</li>
     *   <li>READ COMMITTED makes every statement see what other transactions committed before
     *       it started. So the count below includes the enrollment of whoever held the lock
     *       before us. PostgreSQL uses this level by default. MySQL defaults to REPEATABLE READ,
     *       where the count could reuse a snapshot taken before the lock was granted.</li>
     * </ul>
     */
    @Transactional(isolation = Isolation.READ_COMMITTED)
    public EnrollmentResponse enroll(EnrollmentRequest request) {
        Long studentId = request.studentId();
        Long offeringId = request.offeringId();

        // 1. Verify student exists and is ACTIVE
        Student student = studentRepository.findById(studentId)
                .orElseThrow(() -> new ResourceNotFoundException("Student", "id", studentId));

        if (student.getStatus() != Student.Status.ACTIVE) {
            throw new EnrollmentException(ErrorCode.STUDENT_NOT_ACTIVE);
        }

        // 2. Lock the offering row and check it is open
        CourseOffering offering = offeringRepository.findByIdForUpdate(offeringId)
                .orElseThrow(() -> new ResourceNotFoundException("CourseOffering", "id", offeringId));

        if (!offering.isOpen()) {
            throw new EnrollmentException(ErrorCode.COURSE_CLOSED);
        }

        // 3. Count enrolled students, seeing every enrollment committed before the lock
        int enrolled = enrollmentRepository.countEnrolledByOfferingId(offeringId);

        if (enrolled >= offering.getCapacity()) {
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
        Enrollment enrollment = new Enrollment(student, offering);
        try {
            Enrollment saved = enrollmentRepository.saveAndFlush(enrollment);
            return EnrollmentResponse.from(saved);
        } catch (DataIntegrityViolationException e) {
            // Concurrent request already enrolled this student
            throw new EnrollmentException(ErrorCode.ALREADY_ENROLLED);
        }
    }

    /**
     * Cancel an enrollment.
     * Locks the offering's row, like {@link #enroll}, so a cancellation and an enrollment in the
     * same offering never interleave.
     */
    @Transactional(isolation = Isolation.READ_COMMITTED)
    public EnrollmentResponse cancel(Long enrollmentId) {
        Enrollment enrollment = enrollmentRepository.findById(enrollmentId)
                .orElseThrow(() -> new EnrollmentException(ErrorCode.ENROLLMENT_NOT_FOUND));

        if (enrollment.getStatus() == Enrollment.Status.CANCELLED) {
            throw new EnrollmentException(ErrorCode.ALREADY_CANCELLED);
        }

        offeringRepository.findByIdForUpdate(enrollment.getOffering().getId());

        enrollment.setStatus(Enrollment.Status.CANCELLED);
        Enrollment saved = enrollmentRepository.save(enrollment);
        return EnrollmentResponse.from(saved);
    }
}
