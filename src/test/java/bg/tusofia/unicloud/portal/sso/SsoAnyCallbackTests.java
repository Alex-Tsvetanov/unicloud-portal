package bg.tusofia.unicloud.portal.sso;

import bg.tusofia.unicloud.portal.auth.application.AccessTokenIssuer;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import bg.tusofia.unicloud.portal.user.domain.User;
import jakarta.servlet.Filter;
import jakarta.servlet.http.Cookie;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.mock.web.MockHttpServletResponse;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;
import org.springframework.web.util.UriComponentsBuilder;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.ObjectMapper;

import java.net.URI;
import java.nio.charset.StandardCharsets;
import java.time.LocalDateTime;
import java.util.Base64;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * Staging's rules (app.sso.callback-policy=any): a module may use any http or https callback,
 * registered or not, for login and for logout.
 */
@SpringBootTest(properties = "app.sso.callback-policy=any")
class SsoAnyCallbackTests {

    private static final long STUDENT_ID = 4L;
    private static final long ADMIN_ID = 9L;
    private static final String SESSION_COOKIE = "__Host-UNICLOUD_SSO";

    private final ObjectMapper json = new ObjectMapper();

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
    private AccessTokenIssuer tokenIssuer;

    private MockMvc mockMvc;

    @BeforeEach
    void setUp() {
        mockMvc = MockMvcBuilders.webAppContextSetup(context)
                .addFilters(sessionRepositoryFilter, springSecurityFilterChain)
                .build();
    }

    @Test
    void anyHostCanBeRegistered() throws Exception {
        register("module-" + shortId(), "http://dev-box.lab:3000/auth/callback");
    }

    @Test
    void anUnregisteredCallbackWorksForLoginAndLogout() throws Exception {
        String clientId = "module-" + shortId();
        String secret = register(clientId, "https://registered.example/auth/callback");
        String callback = "http://my-laptop.local:5173/auth/callback";
        String logoutTarget = "http://my-laptop.local:5173/bye";
        Pkce pkce = new Pkce();

        String authorize = UriComponentsBuilder.fromPath("/oauth2/authorize")
                .queryParam("response_type", "code")
                .queryParam("client_id", clientId)
                .queryParam("redirect_uri", callback)
                .queryParam("scope", "openid")
                .queryParam("state", "s")
                .queryParam("code_challenge", pkce.challenge())
                .queryParam("code_challenge_method", "S256")
                .build().encode().toUriString();

        Cookie session = mockMvc.perform(get(URI.create(authorize)).accept(MediaType.TEXT_HTML))
                .andExpect(status().isFound())
                .andReturn().getResponse().getCookie(SESSION_COOKIE);
        MockHttpServletResponse started = mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT"))
                        .cookie(session))
                .andExpect(status().isOk())
                .andReturn().getResponse();
        Cookie loggedIn = started.getCookie(SESSION_COOKIE);
        String continueUrl = json.readTree(started.getContentAsString()).get("continueUrl").asString();

        String redirect = mockMvc.perform(get(URI.create(continueUrl)).cookie(loggedIn).accept(MediaType.TEXT_HTML))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl();
        assertThat(redirect).startsWith(callback + "?code=");
        String code = UriComponentsBuilder.fromUriString(redirect).build().getQueryParams().getFirst("code");

        JsonNode tokens = json.readTree(mockMvc.perform(post("/oauth2/token")
                        .header(HttpHeaders.AUTHORIZATION, "Basic " + Base64.getEncoder().encodeToString(
                                (clientId + ":" + secret).getBytes(StandardCharsets.UTF_8)))
                        .contentType(MediaType.APPLICATION_FORM_URLENCODED)
                        .param("grant_type", "authorization_code")
                        .param("code", code)
                        .param("redirect_uri", callback)
                        .param("code_verifier", pkce.verifier()))
                .andExpect(status().isOk())
                .andReturn().getResponse().getContentAsString());

        String afterLogout = mockMvc.perform(get("/connect/logout")
                        .queryParam("id_token_hint", tokens.get("id_token").asString())
                        .queryParam("client_id", clientId)
                        .queryParam("post_logout_redirect_uri", logoutTarget)
                        .cookie(loggedIn))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl();
        assertThat(afterLogout).isEqualTo(logoutTarget);
    }

    @Test
    void stillNoNonWebCallbacks() throws Exception {
        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("module-" + shortId(), "javascript:alert(1)")))
                .andExpect(status().isBadRequest());
    }

    private String register(String clientId, String redirectUri) throws Exception {
        return json.readTree(mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson(clientId, redirectUri)))
                .andExpect(status().isCreated())
                .andReturn().getResponse().getContentAsString()).get("clientSecret").asString();
    }

    private static String clientJson(String clientId, String redirectUri) {
        return """
                {"clientId": "%s", "name": "Test module", "redirectUris": ["%s"]}
                """.formatted(clientId, redirectUri);
    }

    private String bearer(long userId, String role) {
        LocalDateTime now = LocalDateTime.now();
        User user = User.reconstitute(userId, "someone@tu-sofia.bg", "hash", "Test", "User",
                role, null, true, now, now);
        return "Bearer " + tokenIssuer.issue(user).value();
    }

    private static String shortId() {
        return UUID.randomUUID().toString().substring(0, 8);
    }
}
