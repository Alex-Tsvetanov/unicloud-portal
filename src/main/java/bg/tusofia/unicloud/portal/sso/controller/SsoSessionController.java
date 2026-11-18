package bg.tusofia.unicloud.portal.sso.controller;

import bg.tusofia.unicloud.portal.sso.controller.dto.SsoSessionResponse;
import bg.tusofia.unicloud.portal.sso.infrastructure.SsoSessionMaxAgeFilter;
import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.domain.UserRepository;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import jakarta.servlet.http.HttpSession;
import org.springframework.http.HttpStatus;
import org.springframework.http.ResponseEntity;
import org.springframework.security.authentication.UsernamePasswordAuthenticationToken;
import org.springframework.security.core.annotation.AuthenticationPrincipal;
import org.springframework.security.core.authority.FactorGrantedAuthority;
import org.springframework.security.core.authority.SimpleGrantedAuthority;
import org.springframework.security.core.context.SecurityContext;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.security.web.context.HttpSessionSecurityContextRepository;
import org.springframework.security.web.context.SecurityContextRepository;
import org.springframework.security.web.savedrequest.HttpSessionRequestCache;
import org.springframework.security.web.savedrequest.RequestCache;
import org.springframework.security.web.savedrequest.SavedRequest;
import org.springframework.web.bind.annotation.DeleteMapping;
import org.springframework.web.bind.annotation.PostMapping;
import org.springframework.web.bind.annotation.RequestMapping;
import org.springframework.web.bind.annotation.RestController;

import java.net.URI;
import java.util.List;
import java.util.Optional;

/**
 * Turns a portal login into the SSO browser session, and ends it.
 *
 * <p>The portal logs in with a Bearer token kept in the page, which a browser redirect to
 * /oauth2/authorize cannot send. So when a module sends the user to log in, the login page calls
 * {@code POST} here with that token. The SSO session cookie then carries the login to
 * /oauth2/authorize, and the response says where to continue.
 *
 * <p>{@code POST} needs the Bearer header, so another site cannot call it with the user's cookie
 * (a cross-site request cannot add that header without the portal's CORS consent, which it never
 * gives). The session it creates is used only by the SSO endpoints: the API chain is stateless
 * and never reads it.
 */
@RestController
@RequestMapping("/api/v1/auth/sso-session")
public class SsoSessionController {

    /** The only place the login page may be sent back to. */
    private static final String AUTHORIZE_PATH = "/oauth2/authorize";

    private final RequestCache requestCache = new HttpSessionRequestCache();
    private final SecurityContextRepository contextRepository = new HttpSessionSecurityContextRepository();
    private final UserRepository userRepository;

    public SsoSessionController(UserRepository userRepository) {
        this.userRepository = userRepository;
    }

    @PostMapping
    public ResponseEntity<SsoSessionResponse> start(
            @AuthenticationPrincipal Jwt jwt,
            HttpServletRequest request,
            HttpServletResponse response) {

        User user = parseId(jwt.getSubject())
                .flatMap(userRepository::findById)
                .filter(User::isEnabled)
                .orElse(null);
        if (user == null) {
            return ResponseEntity.status(HttpStatus.UNAUTHORIZED).build();
        }

        String continueUrl = continuation(requestCache.getRequest(request, response));
        requestCache.removeRequest(request, response);

        // A new session id at login, so an id planted in the browser beforehand is worthless
        if (request.getSession(false) != null) {
            request.changeSessionId();
        }

        // The principal's name becomes the "sub" of every module token: the user id, as in the
        // portal's own tokens. The password factor carries the login time, which becomes the ID
        // token's "auth_time": when the user typed the password, i.e. when the portal token was issued.
        SecurityContext context = SecurityContextHolder.getContextHolderStrategy().createEmptyContext();
        context.setAuthentication(UsernamePasswordAuthenticationToken.authenticated(
                String.valueOf(user.getId()),
                null,
                List.of(
                        new SimpleGrantedAuthority("ROLE_" + user.getRole().name()),
                        FactorGrantedAuthority.withAuthority(FactorGrantedAuthority.PASSWORD_AUTHORITY)
                                .issuedAt(jwt.getIssuedAt())
                                .build())));
        contextRepository.saveContext(context, request, response);
        // The session's age counts from the password, not from this call (SsoSessionMaxAgeFilter)
        request.getSession().setAttribute(SsoSessionMaxAgeFilter.LOGIN_AT, jwt.getIssuedAt());

        return ResponseEntity.ok(new SsoSessionResponse(continueUrl));
    }

    /** Ends the SSO session, if there is one. Called by the portal's logout. */
    @DeleteMapping
    public ResponseEntity<Void> end(HttpServletRequest request) {
        HttpSession session = request.getSession(false);
        if (session != null) {
            session.invalidate();
        }
        return ResponseEntity.noContent().build();
    }

    /**
     * The saved /oauth2/authorize request as a path and query, or null. Never an absolute URL and
     * never another path, so the login page cannot be turned into an open redirect.
     */
    private static String continuation(SavedRequest saved) {
        if (saved == null) {
            return null;
        }
        URI uri = URI.create(saved.getRedirectUrl());
        if (!AUTHORIZE_PATH.equals(uri.getRawPath())) {
            return null;
        }
        return uri.getRawQuery() == null ? AUTHORIZE_PATH : AUTHORIZE_PATH + "?" + uri.getRawQuery();
    }

    private static Optional<Long> parseId(String subject) {
        try {
            return Optional.of(Long.parseLong(subject));
        } catch (NumberFormatException e) {
            return Optional.empty();
        }
    }
}
