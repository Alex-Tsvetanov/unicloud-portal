package bg.tusofia.unicloud.portal.student.controller;

import bg.tusofia.unicloud.portal.student.controller.dto.RegisterStudentRequest;
import bg.tusofia.unicloud.portal.student.domain.Student;
import bg.tusofia.unicloud.portal.student.service.StudentRegistrationService;
import jakarta.validation.Valid;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.security.access.prepost.PreAuthorize;
import org.springframework.security.core.Authentication;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/students")
public class StudentRegistrationController {

    private final StudentRegistrationService registrationService;

    public StudentRegistrationController(
            StudentRegistrationService registrationService) {
        this.registrationService = registrationService;
    }

    @PostMapping
    @PreAuthorize("hasRole('INSPECTOR') and hasAuthority('STUDENT_CREATE')")
    public ResponseEntity<?> register(
            @Valid @RequestBody RegisterStudentRequest request,
            Authentication principal) {

        Student student = registrationService.registerStudent(request.email(),
                request.password(),
                request.firstName(),
                request.lastName(),
                request.facultyNumber(),
                request.specialty(),
                request.studyYear(),
                request.studentStatus(),
                principal.getName());

        return ResponseEntity.status(HttpStatus.CREATED)
                .body(Map.of("userId", student.getUserId()));
    }
}
