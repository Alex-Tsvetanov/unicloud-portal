package bg.tusofia.unicloud.portal.repository;

import java.util.List;

import org.springframework.data.jpa.repository.JpaRepository;
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
}
