
package bg.tusofia.unicloud.portal.user.infrastructure.persistence;

import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.infrastructure.persistence.entity.UserJPAEntity;

import java.util.Objects;

public final class UserPersistenceMapper {

    private UserPersistenceMapper() {
        // Utility class
    }

    public static UserJPAEntity toJpaEntity(User user) {
        Objects.requireNonNull(user, "User cannot be null");

        return new UserJPAEntity(
                user.getId(),
                user.getEmail(),
                user.getPasswordHash(),
                user.getFirstName(),
                user.getLastName(),
                user.getRole(),
                user.getPermissionsMask(),
                user.isEnabled()
        );
    }

    public static User toDomain(UserJPAEntity entity) {
        Objects.requireNonNull(entity, "UserJPAEntity cannot be null");

        return User.reconstitute(
                entity.getId(),
                entity.getEmail(),
                entity.getPasswordHash(),
                entity.getFirstName(),
                entity.getLastName(),
                entity.getRole(),
                entity.getPermissionMask(),
                entity.isEnabled(),
                entity.getCreatedAt(),
                entity.getUpdatedAt()
        );
    }
}
