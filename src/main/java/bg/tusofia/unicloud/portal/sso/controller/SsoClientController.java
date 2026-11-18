package bg.tusofia.unicloud.portal.sso.controller;

import bg.tusofia.unicloud.portal.sso.application.SsoClientRegistrationService;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientRequest;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientResponse;
import jakarta.validation.Valid;
import org.springframework.http.HttpStatus;
import org.springframework.security.access.prepost.PreAuthorize;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

/**
 * Registers a module that logs users in through the portal (docs/sso.md, "Register as a
 * client"). Admins only. The response holds the client secret; it is shown once and never stored
 * in a readable form.
 */
@RestController
public class SsoClientController {

    private final SsoClientRegistrationService registrationService;

    public SsoClientController(SsoClientRegistrationService registrationService) {
        this.registrationService = registrationService;
    }

    @PostMapping("/api/v1/sso/clients")
    @ResponseStatus(HttpStatus.CREATED)
    @PreAuthorize("hasRole('ADMIN')")
    public RegisterSsoClientResponse register(
            @Valid @RequestBody RegisterSsoClientRequest request,
            @AuthenticationPrincipal Jwt admin) {

        return registrationService.register(request, admin.getSubject());
    }
}
