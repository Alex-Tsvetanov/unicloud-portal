
package bg.tusofia.unicloud.portal.user.infrastructure.persistence;

import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import bg.tusofia.unicloud.portal.user.infrastructure.persistence.SpringDataUserJpaRepository;
import bg.tusofia.unicloud.portal.user.infrastructure.persistence.entity.UserJPAEntity;
import bg.tusofia.unicloud.portal.user.infrastructure.persistence.UserPersistenceMapper;
import org.springframework.stereotype.Repository;

import java.util.Locale;
import java.util.Objects;
import java.util.Optional;

@Repository
public class UserRepositoryAdapter implements UserRepository {

    private final SpringDataUserJpaRepository jpaRepository;

    public UserRepositoryAdapter(SpringDataUserJpaRepository jpaRepository) {
        this.jpaRepository = jpaRepository;
    }

    @Override
    public User save(User user) {
        Objects.requireNonNull(user, "User cannot be null");

        UserJPAEntity entity = UserPersistenceMapper.toJpaEntity(user);

        UserJPAEntity savedEntity = jpaRepository.saveAndFlush(entity);

        return UserPersistenceMapper.toDomain(savedEntity);
    }

    @Override
    public Optional<User> findById(Long id) {
        return jpaRepository.findById(id)
                .map(UserPersistenceMapper::toDomain);
    }

    @Override
    public Optional<User> findByEmail(String email) {
        return jpaRepository.findByEmail(normalizeEmail(email))
                .map(UserPersistenceMapper::toDomain);
    }

    @Override
    public boolean existsByEmail(String email) {
        return jpaRepository.existsByEmail(normalizeEmail(email));
    }

    private static String normalizeEmail(String email) {
        if (email == null || email.trim().isEmpty()) {
            throw new IllegalArgumentException("Email is required");
        }

        return email.trim().toLowerCase(Locale.ROOT);
    }
}
