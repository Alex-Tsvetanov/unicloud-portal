package bg.tusofia.unicloud.portal.staging;

import bg.tusofia.unicloud.portal.sso.application.SsoClientRegistrationService;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientRequest;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientResponse;
import bg.tusofia.unicloud.portal.staging.dto.CreateStagingUserRequest;
import bg.tusofia.unicloud.portal.staging.dto.SetPermissionsRequest;
import bg.tusofia.unicloud.portal.staging.dto.StagingUserResponse;
import bg.tusofia.unicloud.portal.user.domain.Permission;
import bg.tusofia.unicloud.portal.user.domain.Role;
import jakarta.validation.Valid;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.context.annotation.Profile;
import org.springframework.core.io.ClassPathResource;
import org.springframework.core.io.Resource;
import org.springframework.http.CacheControl;
import org.springframework.http.HttpStatus;
import org.springframework.http.MediaType;
import org.springframework.http.ResponseEntity;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.PathVariable;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.PutMapping;
import org.springframework.web.bind.annotation.RequestBody;
import org.springframework.web.bind.annotation.ResponseStatus;
import org.springframework.web.bind.annotation.RestController;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

/**
 * Staging tools for the other modules' developers (docs/sso.md, "Testing a module against
 * staging"). Exists only with the "staging" profile: on production these routes are not there
 * at all. No login: staging has no users anyone knows the password of, and these routes are how
 * teams make their own.
 *
 * <ul>
 *   <li>{@code GET /staging}: the web page for everything below</li>
 *   <li>{@code GET /api/v1/staging/roles}: roles, their default permissions, every permission</li>
 *   <li>{@code GET /api/v1/staging/users}: every user, with role and permissions</li>
 *   <li>{@code POST /api/v1/staging/users}: create a user with any role and any permissions</li>
 *   <li>{@code PUT /api/v1/staging/users/{id}/permissions}: replace a user's permissions</li>
 *   <li>{@code POST /api/v1/staging/sso-clients}: register a module for single sign-on</li>
 * </ul>
 */
@RestController
@Profile("staging")
public class StagingToolsController {

    private static final Logger log = LoggerFactory.getLogger(StagingToolsController.class);
    private static final Resource PAGE = new ClassPathResource("staging/index.html");

    private final StagingUserService userService;
    private final SsoClientRegistrationService clientRegistrationService;

    public StagingToolsController(
            StagingUserService userService,
            SsoClientRegistrationService clientRegistrationService) {

        this.userService = userService;
        this.clientRegistrationService = clientRegistrationService;
        log.warn("Staging tools are ON: anyone can create users and SSO clients at /staging. "
                + "This must never happen on production.");
    }

    @GetMapping("/staging")
    public ResponseEntity<Resource> page() {
        return ResponseEntity.ok()
                .contentType(MediaType.TEXT_HTML)
                .cacheControl(CacheControl.noStore())
                .body(PAGE);
    }

    @GetMapping("/api/v1/staging/roles")
    public Map<String, Object> roles() {
        Map<String, List<String>> defaults = new LinkedHashMap<>();
        for (Role role : Role.values()) {
            List<String> names = new ArrayList<>();
            Permission.fromMask(role.getDefaultPermissions()).forEach(p -> names.add(p.name()));
            defaults.put(role.name(), names);
        }
        List<String> all = new ArrayList<>();
        for (Permission permission : Permission.values()) {
            all.add(permission.name());
        }
        return Map.of("roleDefaults", defaults, "permissions", all);
    }

    @GetMapping("/api/v1/staging/users")
    public List<StagingUserResponse> users() {
        return userService.list();
    }

    @PostMapping("/api/v1/staging/users")
    @ResponseStatus(HttpStatus.CREATED)
    public StagingUserResponse createUser(@Valid @RequestBody CreateStagingUserRequest request) {
        StagingUserResponse user = userService.create(request);
        log.info("Staging tools: created user {} ({}) with permissions {}", user.email(), user.role(), user.permissions());
        return user;
    }

    @PutMapping("/api/v1/staging/users/{id}/permissions")
    public StagingUserResponse setPermissions(
            @PathVariable long id, @Valid @RequestBody SetPermissionsRequest request) {

        StagingUserResponse user = userService.setPermissions(id, request.permissions());
        log.info("Staging tools: user {} now has permissions {}", user.email(), user.permissions());
        return user;
    }

    @PostMapping("/api/v1/staging/sso-clients")
    @ResponseStatus(HttpStatus.CREATED)
    public RegisterSsoClientResponse registerClient(@Valid @RequestBody RegisterSsoClientRequest request) {
        return clientRegistrationService.register(request, "staging-tools");
    }
}
