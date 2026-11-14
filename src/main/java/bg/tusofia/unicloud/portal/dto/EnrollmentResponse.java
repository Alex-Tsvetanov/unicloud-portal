package bg.tusofia.unicloud.portal.dto;

import java.time.Instant;

import bg.tusofia.unicloud.portal.entity.Enrollment;

public record EnrollmentResponse(
        Long id,
        Long studentId,
        String studentName,
        String facultyNumber,
        Long offeringId,
        String courseCode,
        String courseTitle,
        String semester,
        String status,
        Instant createdAt
) {
    public static EnrollmentResponse from(Enrollment enrollment) {
        return new EnrollmentResponse(
                enrollment.getId(),
                enrollment.getStudent().getUserId(),
                enrollment.getStudent().getUser().getFirstName() + " " + enrollment.getStudent().getUser().getLastName(),
                enrollment.getStudent().getFacultyNumber(),
                enrollment.getOffering().getId(),
                enrollment.getOffering().getCourse().getCode(),
                enrollment.getOffering().getCourse().getTitle(),
                enrollment.getOffering().getSemester(),
                enrollment.getStatus().name(),
                enrollment.getCreatedAt()
        );
    }
}
