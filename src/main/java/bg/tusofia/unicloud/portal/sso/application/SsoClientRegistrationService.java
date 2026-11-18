package bg.tusofia.unicloud.portal.sso.application;

import bg.tusofia.unicloud.portal.exception.DuplicateResourceException;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientRequest;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientResponse;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.security.oauth2.core.AuthorizationGrantType;
import org.springframework.security.oauth2.core.ClientAuthenticationMethod;
import org.springframework.security.oauth2.core.oidc.OidcScopes;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClient;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.settings.ClientSettings;
import org.springframework.security.oauth2.server.authorization.settings.TokenSettings;
import org.springframework.stereotype.Service;

import java.net.URI;
import java.net.URISyntaxException;
import java.security.SecureRandom;
import java.time.Duration;
import java.util.Base64;
import java.util.List;
import java.util.Locale;
import java.util.Set;
import java.util.UUID;

/**
 * Registers a module as an SSO client with the rules of docs/sso.md: exact callback URLs over
 * https (plain http only for localhost), PKCE required, the client secret sent with HTTP Basic,
 * no consent screen (every module is part of UniCloud) and no refresh tokens (a module keeps its
 * own session instead).
 */
@Service
public class SsoClientRegistrationService {

    /** The redirect_uris and post_logout_redirect_uris columns hold at most this many characters. */
    private static final int MAX_URI_LIST_LENGTH = 1000;
    private static final int SECRET_BYTES = 32;
    private static final Set<String> LOOPBACK_HOSTS = Set.of("localhost", "127.0.0.1", "[::1]");

    private static final Logger log = LoggerFactory.getLogger(SsoClientRegistrationService.class);

    private final RegisteredClientRepository clientRepository;
    private final PasswordEncoder passwordEncoder;
    private final Duration codeTimeToLive;
    private final Duration tokenTimeToLive;
    private final String apiAudience;
    private final SecureRandom random = new SecureRandom();

    public SsoClientRegistrationService(
            RegisteredClientRepository clientRepository,
            PasswordEncoder passwordEncoder,
            @Value("${app.sso.code-ttl-seconds}") long codeTtlSeconds,
            @Value("${app.jwt.ttl-minutes}") long tokenTtlMinutes,
            @Value("${app.jwt.audience}") String apiAudience) {

        this.clientRepository = clientRepository;
        this.passwordEncoder = passwordEncoder;
        this.codeTimeToLive = Duration.ofSeconds(codeTtlSeconds);
        this.tokenTimeToLive = Duration.ofMinutes(tokenTtlMinutes);
        this.apiAudience = apiAudience;
    }

    /** {@code registeredBy}: the admin's user id, for the audit log. */
    public RegisterSsoClientResponse register(RegisterSsoClientRequest request, String registeredBy) {
        // A module's tokens carry its client id as "aud". The portal API's own audience must
        // never be one, or the API's audience check would accept that module's tokens.
        if (request.clientId().equalsIgnoreCase(apiAudience)) {
            throw new IllegalArgumentException("clientId: '" + request.clientId() + "' is reserved");
        }
        if (clientRepository.findByClientId(request.clientId()) != null) {
            throw new DuplicateResourceException("SSO client", "clientId", request.clientId());
        }

        List<String> redirectUris = checkedUris(request.redirectUris(), "redirectUris");
        List<String> logoutUris = checkedUris(
                request.postLogoutRedirectUris() == null ? List.of() : request.postLogoutRedirectUris(),
                "postLogoutRedirectUris");
        String secret = newSecret();

        RegisteredClient client = RegisteredClient.withId(UUID.randomUUID().toString())
                .clientId(request.clientId())
                .clientName(request.name().strip())
                .clientSecret(passwordEncoder.encode(secret))
                .clientAuthenticationMethod(ClientAuthenticationMethod.CLIENT_SECRET_BASIC)
                .authorizationGrantType(AuthorizationGrantType.AUTHORIZATION_CODE)
                .redirectUris(uris -> uris.addAll(redirectUris))
                .postLogoutRedirectUris(uris -> uris.addAll(logoutUris))
                .scope(OidcScopes.OPENID)
                .scope(OidcScopes.PROFILE)
                .scope(OidcScopes.EMAIL)
                .scope(SsoUserClaims.UNICLOUD_SCOPE)
                .clientSettings(ClientSettings.builder()
                        .requireProofKey(true)
                        .requireAuthorizationConsent(false)
                        .build())
                .tokenSettings(TokenSettings.builder()
                        .authorizationCodeTimeToLive(codeTimeToLive)
                        .accessTokenTimeToLive(tokenTimeToLive)
                        .build())
                .build();

        clientRepository.save(client);
        log.info("SSO client '{}' registered by user {} with redirect URIs {}",
                client.getClientId(), registeredBy, redirectUris);
        return new RegisterSsoClientResponse(client.getClientId(), secret, redirectUris, logoutUris);
    }

    private static List<String> checkedUris(List<String> uris, String field) {
        List<String> checked = uris.stream().map(String::strip).distinct().toList();
        checked.forEach(uri -> checkUri(uri, field));
        if (String.join(",", checked).length() > MAX_URI_LIST_LENGTH) {
            throw new IllegalArgumentException(
                    field + " must not be longer than " + MAX_URI_LIST_LENGTH + " characters in total");
        }
        return checked;
    }

    /**
     * An absolute https URL (http only for localhost), with no fragment, no user info and no
     * wildcard. The portal compares callbacks character for character, so a wildcard would only
     * ever fail; refusing it early says so.
     */
    private static void checkUri(String value, String field) {
        URI uri;
        try {
            uri = new URI(value);
        } catch (URISyntaxException e) {
            throw new IllegalArgumentException(field + ": not a valid URL: " + value);
        }

        String scheme = uri.getScheme() == null ? "" : uri.getScheme().toLowerCase(Locale.ROOT);
        String host = uri.getHost() == null ? "" : uri.getHost().toLowerCase(Locale.ROOT);
        boolean secure = scheme.equals("https")
                || (scheme.equals("http") && LOOPBACK_HOSTS.contains(host));

        if (!uri.isAbsolute() || host.isEmpty() || !secure) {
            throw new IllegalArgumentException(
                    field + ": must be an https URL (http only for localhost): " + value);
        }
        if (uri.getRawFragment() != null || uri.getRawUserInfo() != null || value.contains("*")
                || value.contains(",")) {
            throw new IllegalArgumentException(
                    field + ": must not contain a fragment, user info, '*' or ',': " + value);
        }
    }

    private String newSecret() {
        byte[] bytes = new byte[SECRET_BYTES];
        random.nextBytes(bytes);
        return Base64.getUrlEncoder().withoutPadding().encodeToString(bytes);
    }
}
