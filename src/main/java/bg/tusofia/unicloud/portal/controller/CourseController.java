package bg.tusofia.unicloud.portal.controller;

import java.net.URI;
import java.util.List;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import bg.tusofia.unicloud.portal.dto.CourseRequest;
import bg.tusofia.unicloud.portal.dto.CourseResponse;
import bg.tusofia.unicloud.portal.service.CourseService;
import jakarta.validation.Valid;

@RestController
@RequestMapping("/api/v1/courses")
public class CourseController {

    private final CourseService courseService;

    public CourseController(CourseService courseService) {
        this.courseService = courseService;
    }

    @GetMapping
    public List<CourseResponse> getAll() {
        return courseService.findAll();
    }

    @GetMapping("/{id}")
    public CourseResponse getById(@PathVariable Integer id) {
        return courseService.findById(id);
    }

    @PostMapping
    public ResponseEntity<CourseResponse> create(@Valid @RequestBody CourseRequest request) {
        CourseResponse created = courseService.create(request);
        URI location = URI.create("/api/v1/courses/" + created.id());
        return ResponseEntity.created(location).body(created);
    }

    @PutMapping("/{id}")
    public CourseResponse update(@PathVariable Integer id, @Valid @RequestBody CourseRequest request) {
        return courseService.update(id, request);
    }
}
