package bg.tusofia.unicloud.portal.sso;

import bg.tusofia.unicloud.portal.sso.infrastructure.SsoSessionMaxAgeFilter;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.Test;
import org.springframework.mock.web.MockFilterChain;
import org.springframework.mock.web.MockHttpServletRequest;
import org.springframework.mock.web.MockHttpServletResponse;
import org.springframework.mock.web.MockHttpSession;
import org.springframework.security.authentication.TestingAuthenticationToken;
import org.springframework.security.core.context.SecurityContextHolder;

import java.time.Clock;
import java.time.Duration;
import java.time.Instant;
import java.time.ZoneOffset;

import static org.assertj.core.api.Assertions.assertThat;

/** The SSO session ends a fixed time after the password, however often it is used. */
class SsoSessionMaxAgeFilterTests {

    private static final Instant NOW = Instant.parse("2026-11-18T12:00:00Z");
    private static final Duration MAX_AGE = Duration.ofHours(8);

    private final SsoSessionMaxAgeFilter filter =
            new SsoSessionMaxAgeFilter(MAX_AGE, Clock.fixed(NOW, ZoneOffset.UTC));

    @AfterEach
    void clearContext() {
        SecurityContextHolder.clearContext();
    }

    @Test
    void sessionOlderThanTheMaximumIsEnded() throws Exception {
        MockHttpSession session = sessionLoggedInAt(NOW.minus(MAX_AGE).minusSeconds(1));
        MockFilterChain chain = new MockFilterChain();

        filter.doFilter(request(session), new MockHttpServletResponse(), chain);

        assertThat(session.isInvalid()).isTrue();
        assertThat(SecurityContextHolder.getContext().getAuthentication()).isNull();
        assertThat(chain.getRequest()).as("the request still goes on, as anonymous").isNotNull();
    }

    @Test
    void youngerSessionIsKept() throws Exception {
        MockHttpSession session = sessionLoggedInAt(NOW.minus(MAX_AGE).plusSeconds(60));

        filter.doFilter(request(session), new MockHttpServletResponse(), new MockFilterChain());

        assertThat(session.isInvalid()).isFalse();
        assertThat(SecurityContextHolder.getContext().getAuthentication()).isNotNull();
    }

    @Test
    void requestWithoutASessionPassesThrough() throws Exception {
        MockHttpServletRequest request = new MockHttpServletRequest();
        MockFilterChain chain = new MockFilterChain();

        filter.doFilter(request, new MockHttpServletResponse(), chain);

        assertThat(request.getSession(false)).isNull();
        assertThat(chain.getRequest()).isNotNull();
    }

    private static MockHttpSession sessionLoggedInAt(Instant loginAt) {
        MockHttpSession session = new MockHttpSession();
        session.setAttribute(SsoSessionMaxAgeFilter.LOGIN_AT, loginAt);
        SecurityContextHolder.getContext().setAuthentication(new TestingAuthenticationToken("4", null, "ROLE_STUDENT"));
        return session;
    }

    private static MockHttpServletRequest request(MockHttpSession session) {
        MockHttpServletRequest request = new MockHttpServletRequest();
        request.setSession(session);
        return request;
    }
}
