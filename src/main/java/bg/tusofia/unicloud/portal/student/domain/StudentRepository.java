package bg.tusofia.unicloud.portal.student.domain;

import java.util.Optional;

public interface StudentRepository {

    Student save(Student student);

    Optional<Student> findById(Long id);

}
