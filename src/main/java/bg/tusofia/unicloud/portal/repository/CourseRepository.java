package bg.tusofia.unicloud.portal.repository;

import java.util.Optional;
import org.springframework.data.jpa.repository.JpaRepository;
import bg.tusofia.unicloud.portal.entity.Course;

public interface CourseRepository extends JpaRepository<Course, Integer> {

    Optional<Course> findByCode(String code);

    boolean existsByCode(String code);
}
