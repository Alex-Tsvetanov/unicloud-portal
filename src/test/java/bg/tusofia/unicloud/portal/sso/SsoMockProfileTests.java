package bg.tusofia.unicloud.portal.sso;

import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import jakarta.servlet.Filter;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;

import static org.assertj.core.api.Assertions.assertThat;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * The local SSO mock (profile "mock", mock/docker-compose.yml) is ready with no setup: a known
 * client, a known password for the test users, any callback, a cookie that works on plain http.
 */
@SpringBootTest
@ActiveProfiles("mock")
class SsoMockProfileTests {

    @DynamicPropertySource
    static void jwtKeys(DynamicPropertyRegistry registry) {
        TestJwtKeys.register(registry);
    }

    @Autowired
    private WebApplicationContext context;

    @Autowired
    @Qualifier("springSecurityFilterChain")
    private Filter springSecurityFilterChain;

    @Autowired
    @Qualifier("springSessionRepositoryFilter")
    private Filter sessionRepositoryFilter;

    @Autowired
    private RegisteredClientRepository clientRepository;

    private MockMvc mockMvc;

    @BeforeEach
    void setUp() {
        mockMvc = MockMvcBuilders.webAppContextSetup(context)
                .addFilters(sessionRepositoryFilter, springSecurityFilterChain)
                .build();
    }

    @Test
    void theKnownClientIsRegistered() {
        assertThat(clientRepository.findByClientId("local-module")).isNotNull();
    }

    @Test
    void testUsersLogInWithTheKnownPassword() throws Exception {
        mockMvc.perform(post("/api/v1/auth/login")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("""
                                {"email": "student1@student.tu-sofia.bg", "password": "password"}
                                """))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.accessToken").exists());
    }

    @Test
    void anyCallbackWorksAndTheCookieSuitsPlainHttp() throws Exception {
        String setCookie = mockMvc.perform(get("/oauth2/authorize")
                        .queryParam("response_type", "code")
                        .queryParam("client_id", "local-module")
                        .queryParam("redirect_uri", "http://localhost:3000/auth/callback")
                        .queryParam("scope", "openid")
                        .queryParam("code_challenge", new Pkce().challenge())
                        .queryParam("code_challenge_method", "S256")
                        .accept(MediaType.TEXT_HTML))
                .andExpect(status().isFound())
                .andReturn().getResponse().getHeader(HttpHeaders.SET_COOKIE);

        assertThat(setCookie).startsWith("UNICLOUD_SSO=").doesNotContain("Secure");
    }

    @Test
    void theIssuerIsLocalhost() throws Exception {
        mockMvc.perform(get("/.well-known/openid-configuration"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.issuer").value("http://localhost:8080"));
    }
}
