package bg.tusofia.unicloud.portal.dto;

import jakarta.validation.constraints.NotNull;

public record EnrollmentRequest(
        @NotNull(message = "Student ID is required")
        Long studentId,

        @NotNull(message = "Offering ID is required")
        Long offeringId
) {
}
