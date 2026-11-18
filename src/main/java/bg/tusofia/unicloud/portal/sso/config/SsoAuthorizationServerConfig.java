package bg.tusofia.unicloud.portal.sso.config;

import bg.tusofia.unicloud.portal.sso.application.SsoUserClaims;
import bg.tusofia.unicloud.portal.sso.infrastructure.SingleUseCodeAuthorizationService;
import bg.tusofia.unicloud.portal.sso.infrastructure.SsoSessionMaxAgeFilter;
import com.nimbusds.jose.jwk.source.JWKSource;
import com.nimbusds.jose.proc.SecurityContext;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.core.annotation.Order;
import org.springframework.http.MediaType;
import org.springframework.jdbc.core.JdbcOperations;
import org.springframework.scheduling.annotation.EnableScheduling;
import org.springframework.security.config.annotation.web.builders.HttpSecurity;
import org.springframework.security.config.annotation.web.configuration.OAuth2AuthorizationServerConfiguration;
import org.springframework.security.oauth2.core.OAuth2AuthenticationException;
import org.springframework.security.oauth2.core.OAuth2Error;
import org.springframework.security.oauth2.core.OAuth2ErrorCodes;
import org.springframework.security.oauth2.core.oidc.OidcUserInfo;
import org.springframework.security.oauth2.core.oidc.endpoint.OidcParameterNames;
import org.springframework.security.oauth2.server.authorization.JdbcOAuth2AuthorizationConsentService;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationConsentService;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationService;
import org.springframework.security.oauth2.server.authorization.client.JdbcRegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.oidc.authentication.OidcUserInfoAuthenticationContext;
import org.springframework.security.oauth2.server.authorization.settings.AuthorizationServerSettings;
import org.springframework.security.oauth2.server.authorization.token.JwtEncodingContext;
import org.springframework.security.oauth2.server.authorization.token.OAuth2TokenCustomizer;
import org.springframework.security.web.SecurityFilterChain;
import org.springframework.security.oauth2.server.resource.web.BearerTokenResolver;
import org.springframework.security.oauth2.server.resource.web.DefaultBearerTokenResolver;
import org.springframework.security.web.authentication.LoginUrlAuthenticationEntryPoint;
import org.springframework.security.web.context.SecurityContextHolderFilter;
import org.springframework.security.web.util.matcher.MediaTypeRequestMatcher;
import org.springframework.session.web.http.CookieSerializer;
import org.springframework.session.web.http.DefaultCookieSerializer;

import java.time.Clock;
import java.time.Duration;
import java.util.LinkedHashMap;
import java.util.Map;

/**
 * Single sign-on for the other UniCloud modules: an OAuth 2.0 / OpenID Connect authorization
 * server on the portal (docs/sso.md). It serves /oauth2/authorize, /oauth2/token, /oauth2/jwks,
 * /.well-known/openid-configuration, /userinfo and /connect/logout.
 *
 * <p>Its filter chain runs before the API chain in SecurityConfig and matches only those
 * endpoints. It is the one place in the portal that uses a browser session (cookie UNICLOUD_SSO,
 * stored in MySQL by Spring Session): /oauth2/authorize is a browser redirect and cannot carry
 * a Bearer token. The login page turns a Bearer login into that session through
 * {@code POST /api/v1/auth/sso-session} (SsoSessionController).
 */
@Configuration
@EnableScheduling
public class SsoAuthorizationServerConfig {

    /** Where /oauth2/authorize sends a browser that has no SSO session yet. */
    public static final String LOGIN_PAGE = "/login.html?sso=1";

    /**
     * The SSO session cookie. The __Host- prefix makes the browser refuse it unless it is Secure,
     * has Path=/ and no Domain, so a sibling subdomain cannot plant one. Spring Session does not
     * read server.servlet.session.cookie.*, hence the explicit serializer below.
     */
    public static final String SESSION_COOKIE = "__Host-UNICLOUD_SSO";

    /** The only SSO endpoint that takes a Bearer token: a module's access token. */
    private static final String USERINFO_PATH = "/userinfo";

    private static final Logger log = LoggerFactory.getLogger(SsoAuthorizationServerConfig.class);

    @Bean
    @Order(1)
    SecurityFilterChain ssoSecurityFilterChain(
            HttpSecurity http,
            JWKSource<SecurityContext> jwkSource,
            SsoUserClaims userClaims,
            @Value("${app.sso.session-max-age}") Duration sessionMaxAge) throws Exception {

        http
                .oauth2AuthorizationServer(server -> {
                    http.securityMatcher(server.getEndpointsMatcher());
                    server.oidc(oidc -> oidc.userInfoEndpoint(userInfo -> userInfo
                            .userInfoMapper(context -> userInfo(context, userClaims))));
                })
                .authorizeHttpRequests(auth -> auth.anyRequest().authenticated())
                // A browser without a session goes to the login page; the request it came with is
                // kept in the session, and the login page resumes it afterwards.
                .exceptionHandling(ex -> ex.defaultAuthenticationEntryPointFor(
                        new LoginUrlAuthenticationEntryPoint(LOGIN_PAGE),
                        new MediaTypeRequestMatcher(MediaType.TEXT_HTML)))
                // /userinfo takes the access token a module received. The library still looks the
                // token up in its own store, so a portal API token is refused there.
                .oauth2ResourceServer(resource -> resource
                        .bearerTokenResolver(userInfoOnly())
                        .jwt(jwt -> jwt.decoder(OAuth2AuthorizationServerConfiguration.jwtDecoder(jwkSource))))
                .addFilterAfter(
                        new SsoSessionMaxAgeFilter(sessionMaxAge, Clock.systemUTC()),
                        SecurityContextHolderFilter.class);

        return http.build();
    }

    /**
     * Bearer tokens count only at /userinfo. Everywhere else in this chain (above all
     * /oauth2/authorize) the user is the one in the SSO session, never a token holder.
     */
    private static BearerTokenResolver userInfoOnly() {
        DefaultBearerTokenResolver resolver = new DefaultBearerTokenResolver();
        return request -> USERINFO_PATH.equals(request.getRequestURI()) ? resolver.resolve(request) : null;
    }

    /**
     * HttpOnly: page scripts never see it. Secure: https only (browsers also allow it on
     * http://localhost). SameSite=Lax: sent when a module redirects the browser here, but not with
     * requests other sites make in the background.
     */
    @Bean
    CookieSerializer ssoSessionCookie() {
        DefaultCookieSerializer cookie = new DefaultCookieSerializer();
        cookie.setCookieName(SESSION_COOKIE);
        cookie.setCookiePath("/");
        cookie.setUseHttpOnlyCookie(true);
        cookie.setUseSecureCookie(true);
        cookie.setSameSite("Lax");
        return cookie;
    }

    @Bean
    RegisteredClientRepository registeredClientRepository(JdbcOperations jdbcOperations) {
        return new JdbcRegisteredClientRepository(jdbcOperations);
    }

    @Bean
    OAuth2AuthorizationService authorizationService(
            JdbcOperations jdbcOperations,
            RegisteredClientRepository registeredClientRepository) {

        return new SingleUseCodeAuthorizationService(jdbcOperations, registeredClientRepository);
    }

    @Bean
    OAuth2AuthorizationConsentService authorizationConsentService(
            JdbcOperations jdbcOperations,
            RegisteredClientRepository registeredClientRepository) {

        return new JdbcOAuth2AuthorizationConsentService(jdbcOperations, registeredClientRepository);
    }

    /** The issuer is fixed when app.sso.issuer is set (production); otherwise taken from the request. */
    @Bean
    AuthorizationServerSettings authorizationServerSettings(
            @Value("${app.sso.issuer:}") String issuer) {

        AuthorizationServerSettings.Builder settings = AuthorizationServerSettings.builder();
        if (issuer.isBlank()) {
            log.warn("APP_SSO_ISSUER is not set: the SSO issuer is taken from each request's host and "
                    + "X-Forwarded-* headers. Set it to the portal's https address outside local development.");
        } else {
            settings.issuer(issuer.strip());
        }
        return settings.build();
    }

    /** Puts the user's claims into every ID token. A user deleted or disabled since login gets none. */
    @Bean
    OAuth2TokenCustomizer<JwtEncodingContext> idTokenClaims(SsoUserClaims userClaims) {
        return context -> {
            if (!OidcParameterNames.ID_TOKEN.equals(context.getTokenType().getValue())) {
                return;
            }
            Map<String, Object> claims = userClaims
                    .forSubject(context.getPrincipal().getName(), context.getAuthorizedScopes())
                    .orElseThrow(SsoAuthorizationServerConfig::unknownUser);
            context.getClaims().claims(existing -> existing.putAll(claims));
        };
    }

    private static OidcUserInfo userInfo(OidcUserInfoAuthenticationContext context, SsoUserClaims userClaims) {
        String subject = context.getAuthorization().getPrincipalName();
        Map<String, Object> claims = new LinkedHashMap<>();
        claims.put("sub", subject);
        claims.putAll(userClaims.forSubject(subject, context.getAuthorization().getAuthorizedScopes())
                .orElseThrow(SsoAuthorizationServerConfig::unknownUser));
        return new OidcUserInfo(claims);
    }

    private static OAuth2AuthenticationException unknownUser() {
        return new OAuth2AuthenticationException(new OAuth2Error(
                OAuth2ErrorCodes.INVALID_GRANT, "The user no longer exists or is disabled", null));
    }
}
