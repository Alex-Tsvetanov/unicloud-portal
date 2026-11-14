package bg.tusofia.unicloud.portal.dto;

import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;
import jakarta.validation.constraints.Size;

public record OfferingRequest(
        @NotNull(message = "Course ID is required")
        Integer courseId,

        @NotBlank(message = "Semester is required")
        @Size(max = 16, message = "Semester must be at most 16 characters")
        String semester,

        @NotNull(message = "Teacher ID is required")
        Long teacherId,

        @NotNull(message = "Capacity is required")
        @Min(value = 1, message = "Capacity must be at least 1")
        Integer capacity,

        Boolean isOpen
) {
    public OfferingRequest {
        if (isOpen == null) {
            isOpen = true;
        }
    }
}
