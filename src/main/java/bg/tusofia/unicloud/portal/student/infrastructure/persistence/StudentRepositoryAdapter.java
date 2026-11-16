package bg.tusofia.unicloud.portal.student.infrastructure.persistence;

import bg.tusofia.unicloud.portal.student.domain.Student;
import bg.tusofia.unicloud.portal.student.domain.StudentRepository;
import bg.tusofia.unicloud.portal.student.infrastructure.persistence.entity.StudentJPAEntity;
import org.springframework.stereotype.Repository;

import java.util.Optional;

@Repository
public class StudentRepositoryAdapter implements StudentRepository {

    private final StudentJpaRepository studentJpaRepository;

    public StudentRepositoryAdapter(StudentJpaRepository studentJpaRepository) {
        this.studentJpaRepository = studentJpaRepository;
    }

    @Override
    public Student save(Student student) {
        StudentJPAEntity entity = StudentPersistenceMapper.toJpaEntity(student);

        StudentJPAEntity savedEntity = studentJpaRepository.save(entity);

        return StudentPersistenceMapper.toDomain(savedEntity);
    }

    @Override
    public Optional<Student> findById(Long id) {
        return studentJpaRepository.findById(id)
                .map(StudentPersistenceMapper::toDomain);
    }
}