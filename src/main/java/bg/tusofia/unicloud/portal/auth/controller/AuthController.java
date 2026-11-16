package bg.tusofia.unicloud.portal.auth.controller;

import bg.tusofia.unicloud.portal.auth.application.AccessToken;
import bg.tusofia.unicloud.portal.auth.application.AuthenticationService;
import bg.tusofia.unicloud.portal.auth.controller.dto.LoginRequest;
import bg.tusofia.unicloud.portal.auth.controller.dto.LoginResponse;
import jakarta.validation.Valid;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.*;

import java.util.Map;

@RestController
@RequestMapping("/api/v1/auth")
public class AuthController {

    private final AuthenticationService authenticationService;

    public AuthController(
            AuthenticationService authenticationService) {
        this.authenticationService = authenticationService;
    }

    @PostMapping("/login")
    public LoginResponse login(
            @Valid @RequestBody LoginRequest request) {

        AccessToken token = authenticationService.authenticate(
                request.email(),
                request.password()
        );

        return new LoginResponse(
                token.value(),
                "Bearer",
                token.expiresInSeconds()
        );
    }

    @GetMapping("/me")
    public Map<String, Object> me(@AuthenticationPrincipal Jwt jwt) {
        return Map.of(
                "userId", jwt.getSubject(),
                "roles", jwt.getClaimAsStringList("roles")
        );
    }
}
