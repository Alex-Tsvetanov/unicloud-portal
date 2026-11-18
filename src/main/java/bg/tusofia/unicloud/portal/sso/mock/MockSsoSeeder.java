package bg.tusofia.unicloud.portal.sso.mock;

import bg.tusofia.unicloud.portal.sso.application.SsoClientRegistrationService;
import bg.tusofia.unicloud.portal.sso.controller.dto.RegisterSsoClientRequest;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.ApplicationArguments;
import org.springframework.boot.ApplicationRunner;
import org.springframework.context.annotation.Profile;
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.security.crypto.password.PasswordEncoder;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.stereotype.Component;

import java.util.List;

/**
 * The local SSO mock (profile "mock" only): makes the portal ready for a module developer with
 * no setup. At every start it gives every user in the database the same known password, and it
 * registers one SSO client with a known id and secret. The test users come from the migrations'
 * test data (V2__test_data.sql).
 *
 * <p>Never active on staging or production: the profile is set only by mock/docker-compose.yml.
 */
@Component
@Profile("mock")
public class MockSsoSeeder implements ApplicationRunner {

    /** A placeholder: the mock accepts any callback URL, registered or not. */
    private static final String PLACEHOLDER_CALLBACK = "http://localhost/callback";

    private static final Logger log = LoggerFactory.getLogger(MockSsoSeeder.class);

    private final JdbcTemplate jdbc;
    private final PasswordEncoder passwordEncoder;
    private final RegisteredClientRepository clientRepository;
    private final SsoClientRegistrationService registrationService;
    private final String clientId;
    private final String clientSecret;
    private final String userPassword;
    private final String issuer;

    public MockSsoSeeder(
            JdbcTemplate jdbc,
            PasswordEncoder passwordEncoder,
            RegisteredClientRepository clientRepository,
            SsoClientRegistrationService registrationService,
            @Value("${app.mock.client-id}") String clientId,
            @Value("${app.mock.client-secret}") String clientSecret,
            @Value("${app.mock.user-password}") String userPassword,
            @Value("${app.sso.issuer}") String issuer) {

        this.jdbc = jdbc;
        this.passwordEncoder = passwordEncoder;
        this.clientRepository = clientRepository;
        this.registrationService = registrationService;
        this.clientId = clientId;
        this.clientSecret = clientSecret;
        this.userPassword = userPassword;
        this.issuer = issuer;
    }

    @Override
    public void run(ApplicationArguments args) {
        jdbc.update("UPDATE users SET password_hash = ?", passwordEncoder.encode(userPassword));

        if (clientRepository.findByClientId(clientId) == null) {
            registrationService.register(
                    new RegisterSsoClientRequest(clientId, "Local module (mock)",
                            List.of(PLACEHOLDER_CALLBACK), List.of()),
                    "mock",
                    clientSecret);
        }

        List<String> emails = jdbc.queryForList("SELECT email FROM users ORDER BY id", String.class);
        log.warn("""

                ==================== UniCloud portal: LOCAL SSO MOCK ====================
                Issuer:        {}
                Client id:     {}
                Client secret: {}
                Users:         {}
                Password:      {} (the same for every user)
                Any callback URL is accepted. Never run this profile on a real server.
                ==========================================================================""",
                issuer, clientId, clientSecret, emails, userPassword);
    }
}
