package bg.tusofia.unicloud.portal.controller;

import java.net.URI;
import java.util.List;

import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PatchMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import bg.tusofia.unicloud.portal.dto.EnrollmentRequest;
import bg.tusofia.unicloud.portal.dto.EnrollmentResponse;
import bg.tusofia.unicloud.portal.service.EnrollmentService;
import jakarta.validation.Valid;

@RestController
@RequestMapping("/api/v1")
public class EnrollmentController {

    private final EnrollmentService enrollmentService;

    public EnrollmentController(EnrollmentService enrollmentService) {
        this.enrollmentService = enrollmentService;
    }

    @PostMapping("/enrollments")
    public ResponseEntity<EnrollmentResponse> enroll(@Valid @RequestBody EnrollmentRequest request) {
        EnrollmentResponse created = enrollmentService.enroll(request);
        URI location = URI.create("/api/v1/enrollments/" + created.id());
        return ResponseEntity.created(location).body(created);
    }

    @GetMapping("/enrollments/{id}")
    public EnrollmentResponse getById(@PathVariable Long id) {
        return enrollmentService.findById(id);
    }

    @PatchMapping("/enrollments/{id}/cancel")
    public EnrollmentResponse cancel(@PathVariable Long id) {
        return enrollmentService.cancel(id);
    }

    @GetMapping("/students/{studentId}/enrollments")
    public List<EnrollmentResponse> getByStudentId(@PathVariable Long studentId) {
        return enrollmentService.findByStudentId(studentId);
    }
}
