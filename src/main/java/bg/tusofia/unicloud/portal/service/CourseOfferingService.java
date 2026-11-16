package bg.tusofia.unicloud.portal.service;

import java.util.List;

import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import bg.tusofia.unicloud.portal.dto.OfferingRequest;
import bg.tusofia.unicloud.portal.dto.OfferingResponse;
import bg.tusofia.unicloud.portal.entity.Course;
import bg.tusofia.unicloud.portal.entity.CourseOffering;
import bg.tusofia.unicloud.portal.entity.User;
import bg.tusofia.unicloud.portal.exception.DuplicateResourceException;
import bg.tusofia.unicloud.portal.exception.ResourceNotFoundException;
import bg.tusofia.unicloud.portal.repository.CourseOfferingRepository;
import bg.tusofia.unicloud.portal.repository.CourseRepository;
import bg.tusofia.unicloud.portal.repository.EnrollmentRepository;
import bg.tusofia.unicloud.portal.repository.UserRepository;

@Service
@Transactional(readOnly = true)
public class CourseOfferingService {

    private final CourseOfferingRepository offeringRepository;
    private final CourseRepository courseRepository;
    private final UserRepository userRepository;
    private final EnrollmentRepository enrollmentRepository;

    public CourseOfferingService(CourseOfferingRepository offeringRepository,
                                  CourseRepository courseRepository,
                                  UserRepository userRepository,
                                  EnrollmentRepository enrollmentRepository) {
        this.offeringRepository = offeringRepository;
        this.courseRepository = courseRepository;
        this.userRepository = userRepository;
        this.enrollmentRepository = enrollmentRepository;
    }

    public List<OfferingResponse> findAll(String semester, Integer courseId, Boolean open) {
        List<CourseOffering> offerings = offeringRepository.findWithFilters(semester, courseId, open);
        return offerings.stream()
                .map(this::toResponse)
                .toList();
    }

    public OfferingResponse findById(Long id) {
        CourseOffering offering = offeringRepository.findById(id)
                .orElseThrow(() -> new ResourceNotFoundException("CourseOffering", "id", id));
        return toResponse(offering);
    }

    @Transactional
    public OfferingResponse create(OfferingRequest request) {
        Course course = courseRepository.findById(request.courseId())
                .orElseThrow(() -> new ResourceNotFoundException("Course", "id", request.courseId()));

        User teacher = userRepository.findById(request.teacherId())
                .orElseThrow(() -> new ResourceNotFoundException("Teacher", "id", request.teacherId()));

        // Check for duplicate: same course + semester
        if (offeringRepository.existsByCourseIdAndSemester(request.courseId(), request.semester())) {
            throw new DuplicateResourceException(
                    "CourseOffering",
                    "course and semester",
                    request.courseId() + " / " + request.semester()
            );
        }

        CourseOffering offering = new CourseOffering(
                course,
                request.semester(),
                teacher,
                request.capacity()
        );
        offering.setOpen(request.isOpen());

        CourseOffering saved = offeringRepository.save(offering);
        return toResponse(saved);
    }

    @Transactional
    public OfferingResponse update(Long id, OfferingRequest request) {
        CourseOffering offering = offeringRepository.findById(id)
                .orElseThrow(() -> new ResourceNotFoundException("CourseOffering", "id", id));

        // Validate capacity is not reduced below enrolled count
        int enrolledCount = enrollmentRepository.countEnrolledByOfferingId(id);
        if (request.capacity() < enrolledCount) {
            throw new IllegalArgumentException(
                    "Cannot reduce capacity to " + request.capacity() +
                    " — " + enrolledCount + " students are already enrolled");
        }

        // Update capacity and open status only (course, semester, teacher are immutable)
        offering.setCapacity(request.capacity());
        offering.setOpen(request.isOpen());

        CourseOffering saved = offeringRepository.save(offering);
        return toResponse(saved);
    }

    private OfferingResponse toResponse(CourseOffering offering) {
        int enrolledCount = enrollmentRepository.countEnrolledByOfferingId(offering.getId());
        return OfferingResponse.from(offering, enrolledCount);
    }
}
