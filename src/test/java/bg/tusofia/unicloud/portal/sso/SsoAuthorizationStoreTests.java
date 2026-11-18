package bg.tusofia.unicloud.portal.sso;

import bg.tusofia.unicloud.portal.sso.infrastructure.SsoAuthorizationCleanup;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.security.oauth2.core.AuthorizationGrantType;
import org.springframework.security.oauth2.core.ClientAuthenticationMethod;
import org.springframework.security.oauth2.core.OAuth2AccessToken;
import org.springframework.security.oauth2.core.OAuth2AuthenticationException;
import org.springframework.security.oauth2.server.authorization.OAuth2Authorization;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationCode;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationService;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClient;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;

import java.time.Duration;
import java.time.Instant;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/**
 * The SSO store's own guarantees, against the real database: a code yields tokens once even when
 * two replicas race, and old logins are deleted.
 */
@SpringBootTest
class SsoAuthorizationStoreTests {

    @DynamicPropertySource
    static void jwtKeys(DynamicPropertyRegistry registry) {
        TestJwtKeys.register(registry);
    }

    @Autowired
    private OAuth2AuthorizationService authorizationService;

    @Autowired
    private RegisteredClientRepository clientRepository;

    @Autowired
    private SsoAuthorizationCleanup cleanup;

    private RegisteredClient client;

    @BeforeEach
    void setUp() {
        client = RegisteredClient.withId(UUID.randomUUID().toString())
                .clientId("store-test-" + UUID.randomUUID().toString().substring(0, 8))
                .clientName("Store test")
                .clientSecret("{noop}" + UUID.randomUUID())
                .clientAuthenticationMethod(ClientAuthenticationMethod.CLIENT_SECRET_BASIC)
                .authorizationGrantType(AuthorizationGrantType.AUTHORIZATION_CODE)
                .redirectUri("https://store-test.example/callback")
                .scope("openid")
                .build();
        clientRepository.save(client);
    }

    @Test
    void twoReplicasExchangingTheSameCodeCannotBothGetTokens() {
        OAuth2Authorization issued = issuedCode();
        OAuth2AuthorizationCode code = issued.getToken(OAuth2AuthorizationCode.class).getToken();

        // Both replicas read the unused code, then each tries to save its own tokens
        OAuth2Authorization replicaA = OAuth2Authorization.from(issued)
                .token(accessToken("token-a")).invalidate(code).build();
        OAuth2Authorization replicaB = OAuth2Authorization.from(issued)
                .token(accessToken("token-b")).invalidate(code).build();

        authorizationService.save(replicaA);
        assertThatThrownBy(() -> authorizationService.save(replicaB))
                .isInstanceOf(OAuth2AuthenticationException.class)
                .hasMessageContaining("already been used");

        // The replay also revokes what the first exchange got: one of the two was not the module
        OAuth2Authorization stored = authorizationService.findById(issued.getId());
        assertThat(stored.getAccessToken().getToken().getTokenValue()).isEqualTo("token-a");
        assertThat(stored.getAccessToken().isInvalidated()).isTrue();
    }

    @Test
    void laterSavesOfTheSameLoginStillWork() {
        OAuth2Authorization issued = issuedCode();
        OAuth2AuthorizationCode code = issued.getToken(OAuth2AuthorizationCode.class).getToken();
        OAuth2Authorization redeemed = OAuth2Authorization.from(issued)
                .token(accessToken("token-a")).invalidate(code).build();
        authorizationService.save(redeemed);

        // For example a logout, which marks the tokens invalid but issues nothing new
        OAuth2Authorization loggedOut = OAuth2Authorization.from(redeemed)
                .invalidate(redeemed.getAccessToken().getToken()).build();
        authorizationService.save(loggedOut);

        assertThat(authorizationService.findById(issued.getId()).getAccessToken().isInvalidated()).isTrue();
    }

    @Test
    void cleanupDeletesOnlyOldLogins() {
        OAuth2Authorization fresh = issuedCode();

        cleanup.deleteOlderThan(Instant.now().minus(Duration.ofHours(1)));
        assertThat(authorizationService.findById(fresh.getId()))
                .as("a login from just now survives a cutoff an hour ago")
                .isNotNull();

        cleanup.deleteOlderThan(Instant.now().plus(Duration.ofHours(1)));
        assertThat(authorizationService.findById(fresh.getId())).isNull();
    }

    private OAuth2Authorization issuedCode() {
        Instant now = Instant.now();
        OAuth2AuthorizationCode code = new OAuth2AuthorizationCode(
                "code-" + UUID.randomUUID(), now, now.plusSeconds(60));
        OAuth2Authorization authorization = OAuth2Authorization.withRegisteredClient(client)
                .id(UUID.randomUUID().toString())
                .principalName("4")
                .authorizationGrantType(AuthorizationGrantType.AUTHORIZATION_CODE)
                .token(code)
                .build();
        authorizationService.save(authorization);
        return authorization;
    }

    private static OAuth2AccessToken accessToken(String value) {
        Instant now = Instant.now();
        return new OAuth2AccessToken(OAuth2AccessToken.TokenType.BEARER, value, now, now.plusSeconds(1800));
    }
}
