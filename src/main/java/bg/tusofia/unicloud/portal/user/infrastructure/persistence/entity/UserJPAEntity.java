
package bg.tusofia.unicloud.portal.user.infrastructure.persistence.entity;

import bg.tusofia.unicloud.portal.user.domain.Role;
import jakarta.persistence.*;
import org.hibernate.annotations.Generated;
import org.hibernate.generator.EventType;

import java.time.LocalDateTime;

@Entity
@Table(name = "users")
public class UserJPAEntity {

    @Id
    @GeneratedValue(strategy = GenerationType.IDENTITY)
    private Long id;

    @Column(name = "email", nullable = false, unique = true, length = 254)
    private String email;

    @Column(name = "password_hash", nullable = false, length = 255)
    private String passwordHash;

    @Column(name = "first_name", nullable = false, length = 100)
    private String firstName;

    @Column(name = "last_name", nullable = false, length = 100)
    private String lastName;

    @Column(name = "role", nullable = false, length = 16)
    private String role;

    @Column(name="permissions_mask", nullable = false)
    private Long permissionMask;

    @Column(name = "enabled", nullable = false)
    private boolean enabled;

    @Generated(event = EventType.INSERT)
    @Column(name = "created_at", nullable = false,
            insertable = false, updatable = false)
    private LocalDateTime createdAt;

    @Generated(event = {EventType.INSERT, EventType.UPDATE})
    @Column(name = "updated_at", nullable = false,
            insertable = false, updatable = false)
    private LocalDateTime updatedAt;

    protected UserJPAEntity() {
        // Required by JPA
    }

    public UserJPAEntity(
            Long id,
            String email,
            String passwordHash,
            String firstName,
            String lastName,
            Role role,
            Long permissionMask,
            boolean enabled) {

        this.id = id;
        this.email = email;
        this.passwordHash = passwordHash;
        this.firstName = firstName;
        this.lastName = lastName;
        this.role = role.toString();
        this.permissionMask = permissionMask;
        this.enabled = enabled;
    }

    public Long getId() { return id; }
    public String getEmail() { return email; }
    public String getPasswordHash() { return passwordHash; }
    public String getFirstName() { return firstName; }
    public String getLastName() { return lastName; }
    public String getRole() { return role; }
    public Long getPermissionMask() {return permissionMask; }
    public boolean isEnabled() { return enabled; }
    public LocalDateTime getCreatedAt() { return createdAt; }
    public LocalDateTime getUpdatedAt() { return updatedAt; }
}
