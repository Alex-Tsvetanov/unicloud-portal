package bg.tusofia.unicloud.portal.staging;

import bg.tusofia.unicloud.portal.exception.DuplicateResourceException;
import bg.tusofia.unicloud.portal.exception.ResourceNotFoundException;
import bg.tusofia.unicloud.portal.staging.dto.CreateStagingUserRequest;
import bg.tusofia.unicloud.portal.staging.dto.StagingUserResponse;
import bg.tusofia.unicloud.portal.student.domain.Student;
import bg.tusofia.unicloud.portal.student.domain.StudentRepository;
import bg.tusofia.unicloud.portal.student.domain.StudentStatus;
import bg.tusofia.unicloud.portal.user.domain.Permission;
import bg.tusofia.unicloud.portal.user.domain.Role;
import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import org.springframework.context.annotation.Profile;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;
import org.springframework.transaction.annotation.Transactional;

import java.util.ArrayList;
import java.util.Collection;
import java.util.EnumSet;
import java.util.List;
import java.util.Locale;
import java.util.Set;

/**
 * Test users for staging (profile "staging" only): any role, any permissions, no login needed.
 * The portal's normal registration checks who may create whom; this deliberately does not.
 */
@Service
@Profile("staging")
public class StagingUserService {

    private static final String LIST_USERS = """
            SELECT u.id, u.email, u.first_name, u.last_name, u.role, u.permissions_mask,
                   s.user_id AS student_id
            FROM users u LEFT JOIN students s ON s.user_id = u.id
            ORDER BY u.id""";

    private final UserRepository userRepository;
    private final StudentRepository studentRepository;
    private final PasswordEncoder passwordEncoder;
    private final JdbcTemplate jdbc;

    public StagingUserService(
            UserRepository userRepository,
            StudentRepository studentRepository,
            PasswordEncoder passwordEncoder,
            JdbcTemplate jdbc) {

        this.userRepository = userRepository;
        this.studentRepository = studentRepository;
        this.passwordEncoder = passwordEncoder;
        this.jdbc = jdbc;
    }

    @Transactional
    public StagingUserResponse create(CreateStagingUserRequest request) {
        Role role = parseRole(request.role());
        if (request.student() != null && role != Role.STUDENT) {
            throw new IllegalArgumentException("student: only a user with role STUDENT has an academic record");
        }
        String email = request.email().strip().toLowerCase(Locale.ROOT);
        if (userRepository.existsByEmail(email)) {
            throw new DuplicateResourceException("User", "email", email);
        }

        User user = User.createNew(email, passwordEncoder.encode(request.password()),
                request.firstName(), request.lastName(), role.name());
        if (request.permissions() != null) {
            setExactly(user, parsePermissions(request.permissions()));
        }
        User saved = userRepository.save(user);

        if (request.student() != null) {
            saveStudent(saved.getId(), request.student());
        }
        return toResponse(saved, request.student() != null);
    }

    @Transactional
    public StagingUserResponse setPermissions(long userId, Collection<String> permissions) {
        User user = userRepository.findById(userId)
                .orElseThrow(() -> new ResourceNotFoundException("User", "id", userId));
        setExactly(user, parsePermissions(permissions));
        User saved = userRepository.save(user);
        return toResponse(saved, studentRepository.findById(userId).isPresent());
    }

    public List<StagingUserResponse> list() {
        return jdbc.query(LIST_USERS, (rs, row) -> new StagingUserResponse(
                rs.getLong("id"),
                rs.getString("email"),
                rs.getString("first_name"),
                rs.getString("last_name"),
                rs.getString("role"),
                names(Permission.fromMask(rs.getLong("permissions_mask"))),
                rs.getObject("student_id") != null));
    }

    private void saveStudent(long userId, CreateStagingUserRequest.StudentRecord record) {
        StudentStatus status = record.status() == null || record.status().isBlank()
                ? StudentStatus.ACTIVE
                : parseStatus(record.status());
        String facultyNumber = record.facultyNumber().strip();
        Integer taken = jdbc.queryForObject(
                "SELECT COUNT(*) FROM students WHERE faculty_number = ?", Integer.class, facultyNumber);
        if (taken != null && taken > 0) {
            throw new DuplicateResourceException("Student", "facultyNumber", facultyNumber);
        }
        studentRepository.save(Student.reconstitute(userId, facultyNumber,
                record.specialty().strip(), record.studyYear(), status));
    }

    private static void setExactly(User user, Set<Permission> wanted) {
        for (Permission permission : Permission.values()) {
            if (wanted.contains(permission)) {
                user.grantPermission(permission);
            } else {
                user.revokePermission(permission);
            }
        }
    }

    private static StagingUserResponse toResponse(User user, boolean student) {
        return new StagingUserResponse(user.getId(), user.getEmail(), user.getFirstName(),
                user.getLastName(), user.getRole().name(),
                names(Permission.fromMask(user.getPermissionsMask())), student);
    }

    private static List<String> names(Set<Permission> permissions) {
        List<String> names = new ArrayList<>();
        permissions.forEach(permission -> names.add(permission.name()));
        return names;
    }

    private static Role parseRole(String value) {
        try {
            return Role.valueOf(value.strip().toUpperCase(Locale.ROOT));
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("role: must be one of " + List.of(Role.values()));
        }
    }

    private static StudentStatus parseStatus(String value) {
        try {
            return StudentStatus.valueOf(value.strip().toUpperCase(Locale.ROOT));
        } catch (IllegalArgumentException e) {
            throw new IllegalArgumentException("student.status: must be one of " + List.of(StudentStatus.values()));
        }
    }

    private static Set<Permission> parsePermissions(Collection<String> values) {
        Set<Permission> permissions = EnumSet.noneOf(Permission.class);
        for (String value : values) {
            try {
                permissions.add(Permission.valueOf(value.strip().toUpperCase(Locale.ROOT)));
            } catch (IllegalArgumentException e) {
                throw new IllegalArgumentException(
                        "permissions: unknown permission '" + value + "', known: " + List.of(Permission.values()));
            }
        }
        return permissions;
    }
}
