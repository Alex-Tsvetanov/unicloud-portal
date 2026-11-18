package bg.tusofia.unicloud.portal.repository;

import java.util.List;
import java.util.Optional;

import jakarta.persistence.LockModeType;

import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Lock;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import bg.tusofia.unicloud.portal.entity.CourseOffering;

public interface CourseOfferingRepository extends JpaRepository<CourseOffering, Long> {

    List<CourseOffering> findBySemester(String semester);

    List<CourseOffering> findByCourseId(Integer courseId);

    List<CourseOffering> findByOpenTrue();

    List<CourseOffering> findBySemesterAndOpenTrue(String semester);

    @Query("SELECT co FROM CourseOffering co WHERE " +
           "(:semester IS NULL OR co.semester = :semester) AND " +
           "(:courseId IS NULL OR co.course.id = :courseId) AND " +
           "(:open IS NULL OR co.open = :open)")
    List<CourseOffering> findWithFilters(
            @Param("semester") String semester,
            @Param("courseId") Integer courseId,
            @Param("open") Boolean open);

    boolean existsByCourseIdAndSemester(Integer courseId, String semester);

    /**
     * Reads the offering and locks its row until the transaction ends. Hibernate writes the
     * locking read for the database in use (SELECT ... FOR UPDATE on MySQL and PostgreSQL), so
     * enrollments in the same offering run one after another.
     */
    @Lock(LockModeType.PESSIMISTIC_WRITE)
    @Query("SELECT co FROM CourseOffering co WHERE co.id = :id")
    Optional<CourseOffering> findByIdForUpdate(@Param("id") Long id);
}
