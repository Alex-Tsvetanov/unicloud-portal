package bg.tusofia.unicloud.portal.user.controller;


import bg.tusofia.unicloud.portal.user.controller.dto.RegisterUserRequest;
import bg.tusofia.unicloud.portal.user.controller.dto.RegisterUserResponse;
import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.service.RegisterUserCommand;
import bg.tusofia.unicloud.portal.user.service.UserRegistrationService;
import jakarta.validation.Valid;

import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.security.access.prepost.PreAuthorize;
import org.springframework.security.core.Authentication;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.web.bind.annotation.*;

@RestController
@RequestMapping("/api/v1/auth")
public class UserRegistrationController {

    private final UserRegistrationService registrationService;

    public UserRegistrationController(
            UserRegistrationService registrationService) {

        this.registrationService = registrationService;
    }

    @PostMapping("/register")
    @PreAuthorize("hasRole('ADMIN') and hasAuthority('STAFF_CREATE')")
    public ResponseEntity<RegisterUserResponse> register(
            @Valid @RequestBody RegisterUserRequest request,
            Authentication authentication) {

        RegisterUserCommand command = new RegisterUserCommand(
                request.email(),
                request.password(),
                request.firstName(),
                request.lastName(),
                request.role()
        );

        String principalEmail = authentication.getName();
        User user = registrationService.register(command, principalEmail);

        return ResponseEntity
                .status(HttpStatus.CREATED)
                .body(new RegisterUserResponse(user.getId()));
    }
}
