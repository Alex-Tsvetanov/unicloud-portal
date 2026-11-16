package bg.tusofia.unicloud.portal.auth.application;

import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import org.springframework.security.authentication.BadCredentialsException;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.stereotype.Service;

import java.util.Locale;

@Service
public class AuthenticationService {

    private final UserRepository userRepository;
    private final PasswordEncoder passwordEncoder;
    private final AccessTokenIssuer tokenIssuer;

    public AuthenticationService(
            UserRepository userRepository,
            PasswordEncoder passwordEncoder,
            AccessTokenIssuer tokenIssuer) {

        this.userRepository = userRepository;
        this.passwordEncoder = passwordEncoder;
        this.tokenIssuer = tokenIssuer;
    }

    public AccessToken authenticate(
            String email,
            String password) {

        String normalizedEmail =
                email.strip().toLowerCase(Locale.ROOT);

        User user = userRepository.findByEmail(normalizedEmail)
                .orElseThrow(() ->
                        new BadCredentialsException(
                                "Invalid credentials"));

        if (!user.isEnabled() ||
                !passwordEncoder.matches(
                        password, user.getPasswordHash())) {

            throw new BadCredentialsException(
                    "Invalid credentials");
        }

        return tokenIssuer.issue(user);
    }
}
