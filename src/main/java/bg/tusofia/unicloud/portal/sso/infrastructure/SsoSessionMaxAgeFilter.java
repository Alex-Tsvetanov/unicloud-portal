package bg.tusofia.unicloud.portal.sso.infrastructure;

import jakarta.servlet.FilterChain;
import jakarta.servlet.ServletException;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;
import jakarta.servlet.http.HttpSession;
import org.springframework.security.core.context.SecurityContextHolder;
import org.springframework.web.filter.OncePerRequestFilter;

import java.io.IOException;
import java.time.Clock;
import java.time.Duration;
import java.time.Instant;

/**
 * Ends an SSO session that is older than the maximum age, counted from when the user typed the
 * password. Spring Session only has an idle timeout, which every module login resets, so without
 * this a browser left logged in on a shared lab PC would stay logged in for good.
 *
 * <p>Runs in the SSO chain right after the security context is loaded. An expired session is
 * invalidated and the context cleared, so the request continues as anonymous and the user is
 * sent to the login page.
 */
public class SsoSessionMaxAgeFilter extends OncePerRequestFilter {

    /** Session attribute: the {@link Instant} the user typed the password. */
    public static final String LOGIN_AT = SsoSessionMaxAgeFilter.class.getName() + ".LOGIN_AT";

    private final Duration maxAge;
    private final Clock clock;

    public SsoSessionMaxAgeFilter(Duration maxAge, Clock clock) {
        this.maxAge = maxAge;
        this.clock = clock;
    }

    @Override
    protected void doFilterInternal(HttpServletRequest request, HttpServletResponse response, FilterChain chain)
            throws ServletException, IOException {

        HttpSession session = request.getSession(false);
        if (session != null && isTooOld(session.getAttribute(LOGIN_AT))) {
            session.invalidate();
            SecurityContextHolder.clearContext();
        }
        chain.doFilter(request, response);
    }

    private boolean isTooOld(Object loginAt) {
        return loginAt instanceof Instant at && at.plus(maxAge).isBefore(clock.instant());
    }
}
