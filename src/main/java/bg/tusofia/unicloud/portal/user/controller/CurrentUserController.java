package bg.tusofia.unicloud.portal.user.controller;

import bg.tusofia.unicloud.portal.student.domain.StudentRepository;
import bg.tusofia.unicloud.portal.user.controller.dto.CurrentUserResponse;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import org.springframework.http.ResponseEntity;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * The profile screen's data: who is logged in and, for a student, the academic record.
 * Only ever the caller's own profile: the user id comes from the token, not the request.
 */
@RestController
public class CurrentUserController {

    private final UserRepository userRepository;
    private final StudentRepository studentRepository;

    public CurrentUserController(
            UserRepository userRepository,
            StudentRepository studentRepository) {

        this.userRepository = userRepository;
        this.studentRepository = studentRepository;
    }

    @GetMapping("/api/v1/users/me")
    public ResponseEntity<CurrentUserResponse> me(@AuthenticationPrincipal Jwt jwt) {
        long userId = Long.parseLong(jwt.getSubject());

        return userRepository.findById(userId)
                .map(user -> ResponseEntity.ok(CurrentUserResponse.of(
                        user,
                        studentRepository.findById(userId).orElse(null))))
                .orElseGet(() -> ResponseEntity.notFound().build());
    }
}
