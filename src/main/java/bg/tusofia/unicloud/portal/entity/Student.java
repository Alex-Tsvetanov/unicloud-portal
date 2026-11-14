package bg.tusofia.unicloud.portal.entity;

import jakarta.persistence.Column;
import jakarta.persistence.Entity;
import jakarta.persistence.EnumType;
import jakarta.persistence.Enumerated;
import jakarta.persistence.FetchType;
import jakarta.persistence.Id;
import jakarta.persistence.JoinColumn;
import jakarta.persistence.MapsId;
import jakarta.persistence.OneToOne;
import jakarta.persistence.Table;

@Entity
@Table(name = "students")
public class Student {

    public enum Status {
        ACTIVE, INTERRUPTED, GRADUATED
    }

    @Id
    @Column(name = "user_id")
    private Long userId;

    @OneToOne(fetch = FetchType.LAZY)
    @MapsId
    @JoinColumn(name = "user_id")
    private User user;

    @Column(name = "faculty_number", nullable = false, unique = true, length = 20)
    private String facultyNumber;

    @Column(nullable = false, length = 200)
    private String specialty;

    @Column(name = "study_year", nullable = false, columnDefinition = "TINYINT UNSIGNED")
    private Short studyYear;

    @Enumerated(EnumType.STRING)
    @Column(nullable = false, length = 16)
    private Status status = Status.ACTIVE;

    protected Student() {
    }

    public Student(User user, String facultyNumber, String specialty, int studyYear) {
        this.user = user;
        this.userId = user.getId();
        this.facultyNumber = facultyNumber;
        this.specialty = specialty;
        this.studyYear = (short) studyYear;
        this.status = Status.ACTIVE;
    }

    public Long getUserId() {
        return userId;
    }

    public User getUser() {
        return user;
    }

    public String getFacultyNumber() {
        return facultyNumber;
    }

    public void setFacultyNumber(String facultyNumber) {
        this.facultyNumber = facultyNumber;
    }

    public String getSpecialty() {
        return specialty;
    }

    public void setSpecialty(String specialty) {
        this.specialty = specialty;
    }

    public Integer getStudyYear() {
        return studyYear.intValue();
    }

    public void setStudyYear(int studyYear) {
        this.studyYear = (short) studyYear;
    }

    public Status getStatus() {
        return status;
    }

    public void setStatus(Status status) {
        this.status = status;
    }
}
