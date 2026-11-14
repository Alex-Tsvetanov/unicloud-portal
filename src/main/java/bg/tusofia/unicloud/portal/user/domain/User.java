package bg.tusofia.unicloud.portal.user.domain;

import java.time.LocalDateTime;
import java.util.Locale;
import java.util.Objects;

public class User {
    private Long id;
    private String email;
    private String passwordHash;
    private String firstName;
    private String lastName;
    private Role role;
    private Long permissionsMask;
    boolean enabled;

    private LocalDateTime createdAt;
    private LocalDateTime updatedAt;

    private User(
            Long id,
            String email,
            String passwordHash,
            String firstName,
            String lastName,
            String role,
            Long permissionsMask,
            boolean enabled,
            LocalDateTime createdAt,
            LocalDateTime updatedAt) {

        this.id = id;
        this.email = normalizeEmail(email);
        this.passwordHash = requireNonBlank(passwordHash, "Password hash");
        this.firstName = requireNonBlank(firstName, "First name");
        this.lastName = requireNonBlank(lastName, "Last name");
        this.role = Role.valueOf(Objects.requireNonNull(role, "Role is required"));
        this.permissionsMask = Role.valueOf(role).getDefaultPermissions();
        this.enabled = enabled;
        this.createdAt = createdAt;
        this.updatedAt = updatedAt;
    }



    public static User createNew(String email, String passwordHash, String firstName, String lastName,
            String role) {

        return new User(null,
                email,
                passwordHash,
                firstName,
                lastName,
                role,
                null,
                true,
                null,
                null);
    }

    public static User reconstitute(
            Long id,
            String email,
            String passwordHash,
            String firstName,
            String lastName,
            String role,
            Long permissionsMask,
            boolean enabled,
            LocalDateTime createdAt,
            LocalDateTime updatedAt) {

        Objects.requireNonNull(id, "Existing user must have an ID");
        Objects.requireNonNull(id, "Existing user must have an ID");
        Objects.requireNonNull(createdAt, "Creation timestamp is required");
        Objects.requireNonNull(updatedAt, "Update timestamp is required");

        return new User(
                id,
                email,
                passwordHash,
                firstName,
                lastName,
                role,
                permissionsMask,
                enabled,
                createdAt,
                updatedAt
        );
    }

    public void enable() {
        this.enabled = true;
    }

    public void disable() {
        this.enabled = false;
    }

    private static String normalizeEmail(String email) {
        return requireNonBlank(email, "Email")
                .toLowerCase(Locale.ROOT);
    }

    private static String requireNonBlank(String value, String field) {
        if (value == null || value.trim().isEmpty()) {
            throw new IllegalArgumentException(field + " is required");
        }
        return value.trim();
    }

    public Long getId() {
        return id;
    }

    public String getEmail() {
        return email;
    }

    public String getPasswordHash() {
        return passwordHash;
    }

    public String getFirstName() {
        return firstName;
    }

    public String getLastName() {
        return lastName;
    }

    public Role getRole() {
        return role;
    }

    public boolean hasPermission(Permission permission) {
        return permission.isGranted(permissionsMask);
    }

    public void grantPermission(Permission permission) {
        permissionsMask |= permission.getMask();
    }

    public void revokePermission(Permission permission) {
        permissionsMask &= ~permission.getMask();
    }

    public long getPermissionsMask() {
        return permissionsMask;
    }

    public boolean isEnabled() {
        return enabled;
    }

    public LocalDateTime getCreatedAt() {
        return createdAt;
    }

    public LocalDateTime getUpdatedAt() {
        return updatedAt;
    }
}
