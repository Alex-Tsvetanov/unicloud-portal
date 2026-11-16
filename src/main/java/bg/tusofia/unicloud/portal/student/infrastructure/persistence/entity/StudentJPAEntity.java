package bg.tusofia.unicloud.portal.student.infrastructure.persistence.entity;

import bg.tusofia.unicloud.portal.student.domain.StudentStatus;
import jakarta.persistence.*;

@Entity
@Table(name = "students")
public class StudentJPAEntity {

    @Id
    @Column(name = "user_id", nullable = false)
    private Long userId;

    @Column(
            name = "faculty_number",
            nullable = false,
            unique = true,
            length = 20
    )
    private String facultyNumber;

    @Column(
            name = "specialty",
            nullable = false,
            length = 200
    )
    private String specialty;

    @Column(
            name = "study_year",
            nullable = false,
            columnDefinition = "TINYINT UNSIGNED"
    )
    private Byte studyYear;

    @Enumerated(EnumType.STRING)
    @Column(
            name = "status",
            nullable = false,
            length = 16
    )
    private StudentStatus status;

    protected StudentJPAEntity() {
        // Required by JPA
    }

    public StudentJPAEntity(
            Long userId,
            String facultyNumber,
            String specialty,
            int studyYear,
            StudentStatus status) {

        this.userId = userId;
        this.facultyNumber = facultyNumber;
        this.specialty = specialty;
        this.studyYear = (byte) studyYear;
        this.status = status;
    }

    public Long getUserId() {
        return userId;
    }

    public String getFacultyNumber() {
        return facultyNumber;
    }

    public String getSpecialty() {
        return specialty;
    }

    public Byte getStudyYear() {
        return studyYear;
    }

    public StudentStatus getStatus() {
        return status;
    }

    public void setStudyYear(int studyYear) {
        this.studyYear = (byte) studyYear;
    }

    public void setStatus(StudentStatus status) {
        this.status = status;
    }
}