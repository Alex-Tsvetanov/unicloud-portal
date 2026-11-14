package bg.tusofia.unicloud.portal.repository;

import java.util.Optional;

import org.springframework.data.jpa.repository.JpaRepository;

import bg.tusofia.unicloud.portal.entity.Student;

public interface StudentRepository extends JpaRepository<Student, Long> {

    Optional<Student> findByFacultyNumber(String facultyNumber);

    boolean existsByFacultyNumber(String facultyNumber);
}
