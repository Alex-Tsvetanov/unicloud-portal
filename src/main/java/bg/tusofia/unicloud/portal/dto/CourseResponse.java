package bg.tusofia.unicloud.portal.dto;

import bg.tusofia.unicloud.portal.entity.Course;

public record CourseResponse(
        Integer id,
        String code,
        String title,
        Integer ectsCredits
) {
    public static CourseResponse from(Course course) {
        return new CourseResponse(
                course.getId(),
                course.getCode(),
                course.getTitle(),
                course.getEctsCredits()
        );
    }
}
