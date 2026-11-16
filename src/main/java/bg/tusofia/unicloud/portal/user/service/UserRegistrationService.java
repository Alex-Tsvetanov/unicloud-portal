package bg.tusofia.unicloud.portal.user.service;

import bg.tusofia.unicloud.portal.user.domain.Role;
import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import jakarta.transaction.Transactional;
import org.springframework.security.access.AccessDeniedException;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;

import java.security.Principal;

@Service
public class UserRegistrationService {

    private final UserRepository userRepository;

    private final PasswordEncoder passwordEncoder;


    public UserRegistrationService(UserRepository userRepository, PasswordEncoder passwordEncoder) {
        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
    }

    @Transactional
    public User register(
            RegisterUserCommand registerUserCommand,
            String principalId) {

        User principal = userRepository.findById(Long.valueOf(principalId)).orElseThrow( () ->
                new IllegalArgumentException("Invalid principal"));

        checkUserRegistrationAllowed(registerUserCommand, principal);

        if (userRepository.existsByEmail(registerUserCommand.email())) {
            throw new IllegalArgumentException("Email already registered");
        }


        User user = User.createNew(
                registerUserCommand.email(),
                passwordEncoder.encode(registerUserCommand.password()),
                registerUserCommand.firstName(),
                registerUserCommand.lastName(),
                registerUserCommand.role()
        );

        return userRepository.save(user);
    }

    private void checkUserRegistrationAllowed(RegisterUserCommand command, User principal) {
        Role principalRole = principal.getRole();
        Role targetRole = Role.valueOf(command.role());

        boolean allowed = switch (principalRole) {
            case ADMIN -> targetRole == Role.ADMIN
                    || targetRole == Role.TEACHER
                    || targetRole == Role.INSPECTOR;

            case INSPECTOR -> targetRole == Role.STUDENT;

            default -> false;
        };

        if (!allowed) {
            throw new AccessDeniedException(
                    "You are not allowed to register users with role: " + targetRole
            );
        }
    }
}
