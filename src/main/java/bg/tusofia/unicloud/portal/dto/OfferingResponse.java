package bg.tusofia.unicloud.portal.dto;

import bg.tusofia.unicloud.portal.entity.CourseOffering;

public record OfferingResponse(
        Long id,
        Integer courseId,
        String courseCode,
        String courseTitle,
        String semester,
        Long teacherId,
        String teacherName,
        Integer capacity,
        Integer enrolledCount,
        Integer availableSeats,
        Boolean isOpen
) {
    public static OfferingResponse from(CourseOffering offering, int enrolledCount) {
        return new OfferingResponse(
                offering.getId(),
                offering.getCourse().getId(),
                offering.getCourse().getCode(),
                offering.getCourse().getTitle(),
                offering.getSemester(),
                offering.getTeacher().getId(),
                offering.getTeacher().getFirstName() + " " + offering.getTeacher().getLastName(),
                offering.getCapacity(),
                enrolledCount,
                offering.getCapacity() - enrolledCount,
                offering.isOpen()
        );
    }
}
