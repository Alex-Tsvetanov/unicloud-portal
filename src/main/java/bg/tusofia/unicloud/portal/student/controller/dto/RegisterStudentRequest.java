package bg.tusofia.unicloud.portal.student.controller.dto;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

public record RegisterStudentRequest(
        @NotBlank
        @Email
        String email,

        @NotBlank
        String password,

        @NotBlank
        String firstName,

        @NotBlank
        String lastName,

        @NotBlank
        String facultyNumber,

        @NotBlank
        String specialty,

        @NotNull
        Integer studyYear,

        @NotBlank
        String studentStatus) {
}
