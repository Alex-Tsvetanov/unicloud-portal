package bg.tusofia.unicloud.portal.service;

import java.util.List;

import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import bg.tusofia.unicloud.portal.dto.CourseRequest;
import bg.tusofia.unicloud.portal.dto.CourseResponse;
import bg.tusofia.unicloud.portal.entity.Course;
import bg.tusofia.unicloud.portal.exception.DuplicateResourceException;
import bg.tusofia.unicloud.portal.exception.ResourceNotFoundException;
import bg.tusofia.unicloud.portal.repository.CourseRepository;

@Service
@Transactional(readOnly = true)
public class CourseService {

    private final CourseRepository courseRepository;

    public CourseService(CourseRepository courseRepository) {
        this.courseRepository = courseRepository;
    }

    public List<CourseResponse> findAll() {
        return courseRepository.findAll().stream()
                .map(CourseResponse::from)
                .toList();
    }

    public CourseResponse findById(Integer id) {
        Course course = courseRepository.findById(id)
                .orElseThrow(() -> new ResourceNotFoundException("Course", "id", id));
        return CourseResponse.from(course);
    }

    public CourseResponse findByCode(String code) {
        Course course = courseRepository.findByCode(code)
                .orElseThrow(() -> new ResourceNotFoundException("Course", "code", code));
        return CourseResponse.from(course);
    }

    @Transactional
    public CourseResponse create(CourseRequest request) {
        if (courseRepository.existsByCode(request.code())) {
            throw new DuplicateResourceException("Course", "code", request.code());
        }

        Course course = new Course(
                request.code(),
                request.title(),
                request.ectsCredits()
        );

        Course saved = courseRepository.save(course);
        return CourseResponse.from(saved);
    }

    @Transactional
    public CourseResponse update(Integer id, CourseRequest request) {
        Course course = courseRepository.findById(id)
                .orElseThrow(() -> new ResourceNotFoundException("Course", "id", id));

        // Check if new code conflicts with another course
        courseRepository.findByCode(request.code())
                .filter(existing -> !existing.getId().equals(id))
                .ifPresent(existing -> {
                    throw new DuplicateResourceException("Course", "code", request.code());
                });

        course.setCode(request.code());
        course.setTitle(request.title());
        course.setEctsCredits(request.ectsCredits());

        Course saved = courseRepository.save(course);
        return CourseResponse.from(saved);
    }
}
