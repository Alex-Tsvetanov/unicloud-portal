package bg.tusofia.unicloud.portal.repository;

import java.util.List;
import java.util.Optional;

import org.springframework.data.jpa.repository.JpaRepository;
import org.springframework.data.jpa.repository.Query;
import org.springframework.data.repository.query.Param;

import bg.tusofia.unicloud.portal.entity.Enrollment;

public interface EnrollmentRepository extends JpaRepository<Enrollment, Long> {

    List<Enrollment> findByStudentUserId(Long studentId);

    List<Enrollment> findByOfferingId(Long offeringId);

    Optional<Enrollment> findByStudentUserIdAndOfferingId(Long studentId, Long offeringId);

    @Query("SELECT COUNT(e) FROM Enrollment e WHERE e.offering.id = :offeringId AND e.status = 'ENROLLED'")
    int countEnrolledByOfferingId(@Param("offeringId") Long offeringId);

    boolean existsByStudentUserIdAndOfferingId(Long studentId, Long offeringId);
}
