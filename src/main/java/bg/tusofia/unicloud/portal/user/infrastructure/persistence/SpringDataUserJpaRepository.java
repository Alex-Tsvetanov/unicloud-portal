package bg.tusofia.unicloud.portal.user.infrastructure.persistence;

import bg.tusofia.unicloud.portal.user.infrastructure.persistence.entity.UserJPAEntity;
import org.springframework.data.jpa.repository.JpaRepository;

import java.util.Optional;

public interface SpringDataUserJpaRepository
        extends JpaRepository<UserJPAEntity, Long> {

    Optional<UserJPAEntity> findByEmail(String email);

    boolean existsByEmail(String email);
}
