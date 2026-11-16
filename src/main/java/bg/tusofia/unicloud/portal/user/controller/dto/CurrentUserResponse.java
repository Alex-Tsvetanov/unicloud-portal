package bg.tusofia.unicloud.portal.user.controller.dto;

import bg.tusofia.unicloud.portal.student.domain.Student;
import bg.tusofia.unicloud.portal.user.domain.User;

/** The logged-in user's profile; {@code student} is null for staff. */
public record CurrentUserResponse(
        Long id,
        String email,
        String firstName,
        String lastName,
        String role,
        StudentProfile student
) {

    public record StudentProfile(
            String facultyNumber,
            String specialty,
            int studyYear,
            String status
    ) {
    }

    public static CurrentUserResponse of(User user, Student student) {
        StudentProfile profile = student == null ? null : new StudentProfile(
                student.getFacultyNumber(),
                student.getSpecialty(),
                student.getStudyYear(),
                student.getStatus().name());

        return new CurrentUserResponse(
                user.getId(),
                user.getEmail(),
                user.getFirstName(),
                user.getLastName(),
                user.getRole().name(),
                profile);
    }
}
