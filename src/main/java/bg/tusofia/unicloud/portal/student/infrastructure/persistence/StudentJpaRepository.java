package bg.tusofia.unicloud.portal.student.infrastructure.persistence;

import bg.tusofia.unicloud.portal.student.infrastructure.persistence.entity.StudentJPAEntity;
import org.springframework.data.jpa.repository.JpaRepository;

public interface StudentJpaRepository
        extends JpaRepository<StudentJPAEntity, Long> {
}