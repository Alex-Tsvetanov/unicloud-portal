package bg.tusofia.unicloud.portal.dto;

import jakarta.validation.constraints.Max;
import jakarta.validation.constraints.Min;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.Size;

public record CourseRequest(
        @NotBlank(message = "Course code is required")
        @Size(max = 16, message = "Course code must be at most 16 characters")
        String code,

        @NotBlank(message = "Course title is required")
        @Size(max = 200, message = "Course title must be at most 200 characters")
        String title,

        @Min(value = 1, message = "ECTS credits must be at least 1")
        @Max(value = 30, message = "ECTS credits must be at most 30")
        Integer ectsCredits
) {
}
