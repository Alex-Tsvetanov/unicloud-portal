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
import org.springframework.jdbc.core.JdbcTemplate;
import org.springframework.security.oauth2.core.AuthorizationGrantType;
import org.springframework.security.oauth2.core.ClientAuthenticationMethod;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClient;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;
import org.springframework.security.oauth2.server.authorization.settings.ClientSettings;
import org.springframework.mock.web.MockHttpServletResponse;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.MvcResult;
import org.springframework.test.web.servlet.request.MockHttpServletRequestBuilder;
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
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.delete;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.header;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * Single sign-on end to end, as a module would use it (docs/sso.md): register the module, send
 * the browser to /oauth2/authorize, log in on the portal, get the code at the callback, exchange
 * it server to server, read the ID token and /userinfo, log out.
 *
 * <p>Requests go through Spring Session's filter and the real security filter chains, so the SSO
 * session is stored in the database exactly as in production. Users come from V2__test_data.sql.
 */
/*
 * Production rules: callbacks must be registered and on an allowed host. The test modules live
 * under *.example; localhost is listed only to test the http exception.
 */
@SpringBootTest(properties = "app.sso.allowed-callback-hosts=*.example,localhost")
class SsoFlowTests {

    /** student1@student.tu-sofia.bg: an active student with faculty number 121219001. */
    private static final long STUDENT_ID = 4L;
    /** admin@tu-sofia.bg */
    private static final long ADMIN_ID = 9L;
    /** student5@student.tu-sofia.bg, disabled for a moment by one test and enabled again. */
    private static final long DISABLED_LATER_ID = 8L;

    private static final String SESSION_COOKIE = "__Host-UNICLOUD_SSO";
    private static final String STATE = "state-123";
    private static final String NONCE = "nonce-456";

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

    @Autowired
    private JdbcTemplate jdbc;

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
    void moduleLogsTheUserInThroughThePortal() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();

        // 1. The module sends the browser to the portal; without a session it lands on the login page
        MockHttpServletResponse toLogin = mockMvc.perform(authorize(client, pkce))
                .andExpect(status().isFound())
                .andReturn().getResponse();
        assertThat(toLogin.getRedirectedUrl()).endsWith("/login.html?sso=1");
        Cookie session = toLogin.getCookie(SESSION_COOKIE);
        assertThat(session).isNotNull();
        assertThat(toLogin.getHeader(HttpHeaders.SET_COOKIE))
                .contains("HttpOnly").contains("Secure").contains("SameSite=Lax");

        // 2. The login page turns its portal login into the SSO session and learns where to continue
        MockHttpServletResponse started = mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT"))
                        .cookie(session))
                .andExpect(status().isOk())
                .andReturn().getResponse();
        String continueUrl = json.readTree(started.getContentAsString()).get("continueUrl").asString();
        assertThat(continueUrl).startsWith("/oauth2/authorize?");
        Cookie loggedIn = started.getCookie(SESSION_COOKIE);
        assertThat(loggedIn).isNotNull();
        assertThat(loggedIn.getValue()).as("a new session id at login").isNotEqualTo(session.getValue());

        // 3. Back at /oauth2/authorize the portal sends the code to the registered callback
        String callback = mockMvc.perform(get(URI.create(continueUrl)).cookie(loggedIn).accept(MediaType.TEXT_HTML))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl();
        assertThat(callback).startsWith(client.redirectUri() + "?");
        var params = UriComponentsBuilder.fromUriString(callback).build().getQueryParams();
        assertThat(params.getFirst("state")).isEqualTo(STATE);
        String code = params.getFirst("code");
        assertThat(code).isNotBlank();

        // 4. The module's backend exchanges the code
        JsonNode tokens = json.readTree(mockMvc.perform(exchange(client, code, pkce.verifier()))
                .andExpect(status().isOk())
                .andReturn().getResponse().getContentAsString());

        JsonNode idToken = payload(tokens.get("id_token").asString());
        assertThat(idToken.get("sub").asString()).isEqualTo(String.valueOf(STUDENT_ID));
        assertThat(idToken.get("aud").toString()).contains(client.clientId());
        assertThat(idToken.get("nonce").asString()).isEqualTo(NONCE);
        assertThat(idToken.get("email").asString()).isEqualTo("student1@student.tu-sofia.bg");
        assertThat(idToken.get("role").asString()).isEqualTo("STUDENT");
        assertThat(idToken.get("permissions").toString()).contains("ENROLLMENT_REQUEST_CREATE");
        assertThat(idToken.get("student").get("faculty_number").asString()).isEqualTo("121219001");

        // 5. The module's access token reads /userinfo, but not the portal's own API
        String accessToken = tokens.get("access_token").asString();
        mockMvc.perform(get("/userinfo").header(HttpHeaders.AUTHORIZATION, "Bearer " + accessToken))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.sub").value(String.valueOf(STUDENT_ID)))
                .andExpect(jsonPath("$.family_name").value("Стоянов"))
                .andExpect(jsonPath("$.student.study_year").value(2));
        mockMvc.perform(get("/api/v1/users/me").header(HttpHeaders.AUTHORIZATION, "Bearer " + accessToken))
                .andExpect(status().isUnauthorized());

        // 6. The code works once only. Replaying it also revokes the tokens it gave, in case the
        //    first exchange was the attacker's
        mockMvc.perform(exchange(client, code, pkce.verifier()))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.error").value("invalid_grant"));
        mockMvc.perform(get("/userinfo").header(HttpHeaders.AUTHORIZATION, "Bearer " + accessToken))
                .andExpect(status().isUnauthorized());

        // 7. A second module login while the SSO session lives needs no password
        Pkce second = new Pkce();
        assertThat(mockMvc.perform(authorize(client, second).cookie(loggedIn))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl()).startsWith(client.redirectUri() + "?code=");

        // 8. The portal's logout ends the SSO session: the next module login asks for the password
        mockMvc.perform(delete("/api/v1/auth/sso-session").cookie(loggedIn))
                .andExpect(status().isNoContent());
        assertThat(mockMvc.perform(authorize(client, new Pkce()).cookie(loggedIn))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl()).endsWith("/login.html?sso=1");
    }

    @Test
    void moduleLogoutEndsTheSsoSession() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        Cookie session = mockMvc.perform(authorize(client, pkce))
                .andReturn().getResponse().getCookie(SESSION_COOKIE);
        MockHttpServletResponse started = mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT"))
                        .cookie(session))
                .andReturn().getResponse();
        Cookie loggedIn = started.getCookie(SESSION_COOKIE);
        String continueUrl = json.readTree(started.getContentAsString()).get("continueUrl").asString();
        String callback = mockMvc.perform(get(URI.create(continueUrl)).cookie(loggedIn).accept(MediaType.TEXT_HTML))
                .andReturn().getResponse().getRedirectedUrl();
        String code = UriComponentsBuilder.fromUriString(callback).build().getQueryParams().getFirst("code");
        String idToken = json.readTree(mockMvc.perform(exchange(client, code, pkce.verifier()))
                .andReturn().getResponse().getContentAsString()).get("id_token").asString();

        // The module sends the browser to the portal's logout with the ID token it holds
        String afterLogout = mockMvc.perform(get("/connect/logout")
                        .queryParam("id_token_hint", idToken)
                        .queryParam("client_id", client.clientId())
                        .queryParam("post_logout_redirect_uri", logoutUri(client.redirectUri()))
                        .queryParam("state", STATE)
                        .cookie(loggedIn))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl();
        assertThat(afterLogout).isEqualTo(logoutUri(client.redirectUri()) + "?state=" + STATE);

        // The next module login asks for the password again
        assertThat(mockMvc.perform(authorize(client, new Pkce()).cookie(loggedIn))
                .andReturn().getResponse().getRedirectedUrl()).endsWith("/login.html?sso=1");
    }

    @Test
    void codeNeedsTheVerifierThatStartedTheLogin() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        String code = loginAndGetCode(client, pkce);

        mockMvc.perform(exchange(client, code, new Pkce().verifier()))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.error").value("invalid_grant"));
    }

    @Test
    void codeNeedsTheClientSecret() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        String code = loginAndGetCode(client, pkce);
        Client wrongSecret = new Client(client.clientId(), "not-the-secret", client.redirectUri());

        mockMvc.perform(exchange(wrongSecret, code, pkce.verifier()))
                .andExpect(status().isUnauthorized())
                .andExpect(jsonPath("$.error").value("invalid_client"));
    }

    @Test
    void unregisteredCallbackIsRefusedWithoutARedirect() throws Exception {
        Client client = registerClient();
        Client attacker = new Client(client.clientId(), client.secret(), "https://evil.example/callback");

        MvcResult result = mockMvc.perform(authorize(attacker, new Pkce()))
                .andExpect(status().isBadRequest())
                .andReturn();
        assertThat(result.getResponse().getRedirectedUrl()).isNull();
    }

    @Test
    void callbackMustMatchExactly() throws Exception {
        Client client = registerClient();
        Client longer = new Client(client.clientId(), client.secret(), client.redirectUri() + "/../admin");

        mockMvc.perform(authorize(longer, new Pkce()))
                .andExpect(status().isBadRequest());
    }

    @Test
    void callbackOnAHostNoLongerAllowedIsRefused() throws Exception {
        // Registered back when its host was allowed; the host has since left the config file
        String clientId = "module-" + shortId();
        String redirectUri = "https://" + clientId + ".elsewhere.org/auth/callback";
        clientRepository.save(RegisteredClient.withId(UUID.randomUUID().toString())
                .clientId(clientId)
                .clientSecret("{noop}" + UUID.randomUUID())
                .clientAuthenticationMethod(ClientAuthenticationMethod.CLIENT_SECRET_BASIC)
                .authorizationGrantType(AuthorizationGrantType.AUTHORIZATION_CODE)
                .redirectUri(redirectUri)
                .scope("openid")
                .clientSettings(ClientSettings.builder().requireProofKey(true).build())
                .build());
        Client client = new Client(clientId, "unused", redirectUri);

        MvcResult result = mockMvc.perform(authorize(client, new Pkce(), "openid"))
                .andExpect(status().isBadRequest())
                .andReturn();
        assertThat(result.getResponse().getRedirectedUrl()).isNull();
    }

    @Test
    void callbackHostMustBeInTheConfigFile() throws Exception {
        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("module-" + shortId(), "https://module.elsewhere.org/auth/callback")))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.message").value(org.hamcrest.Matchers.containsString("sso-callback-hosts")));
    }

    @Test
    void loginWithoutPkceIsRefused() throws Exception {
        Client client = registerClient();
        String withoutPkce = "/oauth2/authorize?response_type=code&scope=openid&state=" + STATE
                + "&client_id=" + client.clientId() + "&redirect_uri=" + client.redirectUri();

        String location = mockMvc.perform(get(withoutPkce).accept(MediaType.TEXT_HTML))
                .andExpect(status().isFound())
                .andReturn().getResponse().getRedirectedUrl();
        assertThat(location).startsWith(client.redirectUri() + "?error=invalid_request");
    }

    @Test
    void moduleGetsOnlyTheClaimsItsScopesAllow() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        Cookie session = mockMvc.perform(authorize(client, pkce, "openid email"))
                .andReturn().getResponse().getCookie(SESSION_COOKIE);
        MockHttpServletResponse started = mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT"))
                        .cookie(session))
                .andReturn().getResponse();
        String continueUrl = json.readTree(started.getContentAsString()).get("continueUrl").asString();
        String callback = mockMvc.perform(get(URI.create(continueUrl))
                        .cookie(started.getCookie(SESSION_COOKIE)).accept(MediaType.TEXT_HTML))
                .andReturn().getResponse().getRedirectedUrl();
        String code = UriComponentsBuilder.fromUriString(callback).build().getQueryParams().getFirst("code");

        JsonNode idToken = payload(json.readTree(mockMvc.perform(exchange(client, code, pkce.verifier()))
                .andExpect(status().isOk())
                .andReturn().getResponse().getContentAsString()).get("id_token").asString());
        assertThat(idToken.get("email").asString()).isEqualTo("student1@student.tu-sofia.bg");
        assertThat(idToken.has("family_name")).as("profile not asked for").isFalse();
        assertThat(idToken.has("role")).as("unicloud not asked for").isFalse();
        assertThat(idToken.has("student")).as("unicloud not asked for").isFalse();
    }

    @Test
    void aBearerTokenCannotStandInForTheSsoSession() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        String code = loginAndGetCode(client, pkce);
        String moduleAccessToken = json.readTree(mockMvc.perform(exchange(client, code, pkce.verifier()))
                .andExpect(status().isOk())
                .andReturn().getResponse().getContentAsString()).get("access_token").asString();

        // Neither the portal's own token nor a module token gets a code: only the SSO session does
        for (String token : new String[] {bearer(STUDENT_ID, "STUDENT"), "Bearer " + moduleAccessToken}) {
            assertThat(mockMvc.perform(authorize(client, new Pkce()).header(HttpHeaders.AUTHORIZATION, token))
                    .andExpect(status().isFound())
                    .andReturn().getResponse().getRedirectedUrl()).endsWith("/login.html?sso=1");
        }
    }

    @Test
    void theApiAudienceCannotBeAClientId() throws Exception {
        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("unicloud-api", "https://api.example/auth/callback")))
                .andExpect(status().isBadRequest());
    }

    @Test
    void userDisabledAfterLoginGetsNoTokens() throws Exception {
        Client client = registerClient();
        Pkce pkce = new Pkce();
        String code = loginAndGetCode(client, pkce, DISABLED_LATER_ID);

        jdbc.update("UPDATE users SET enabled = FALSE WHERE id = ?", DISABLED_LATER_ID);
        try {
            mockMvc.perform(exchange(client, code, pkce.verifier()))
                    .andExpect(status().isBadRequest())
                    .andExpect(jsonPath("$.error").value("invalid_grant"))
                    .andExpect(jsonPath("$.id_token").doesNotExist());
        } finally {
            jdbc.update("UPDATE users SET enabled = TRUE WHERE id = ?", DISABLED_LATER_ID);
        }
    }

    @Test
    void unknownUserGetsNoSsoSession() throws Exception {
        mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(424242L, "STUDENT")))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void ssoSessionIsNotStartedWithoutAPortalLogin() throws Exception {
        mockMvc.perform(post("/api/v1/auth/sso-session"))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void ssoSessionWithNothingWaitingHasNoContinueUrl() throws Exception {
        mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT")))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.continueUrl").doesNotExist());
    }

    @Test
    void onlyAnAdminRegistersAClient() throws Exception {
        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("module-" + shortId(), "https://module.example/auth/callback")))
                .andExpect(status().isForbidden());
    }

    @Test
    void callbackMustBeHttpsExceptOnLocalhost() throws Exception {
        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("module-" + shortId(), "http://module.example/auth/callback")))
                .andExpect(status().isBadRequest());

        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson("module-" + shortId(), "http://localhost:3000/auth/callback")))
                .andExpect(status().isCreated());
    }

    @Test
    void clientIdIsRegisteredOnce() throws Exception {
        Client client = registerClient();

        mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson(client.clientId(), client.redirectUri())))
                .andExpect(status().isConflict());
    }

    @Test
    void discoveryAndKeysArePublic() throws Exception {
        mockMvc.perform(get("/.well-known/openid-configuration"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.issuer").exists())
                .andExpect(jsonPath("$.authorization_endpoint").exists())
                .andExpect(jsonPath("$.token_endpoint").exists())
                .andExpect(jsonPath("$.userinfo_endpoint").exists())
                .andExpect(jsonPath("$.end_session_endpoint").exists());

        mockMvc.perform(get("/oauth2/jwks"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.keys[0].kid").exists())
                .andExpect(jsonPath("$.keys[0].d").doesNotExist());
    }

    @Test
    void portalTokenIsRefusedAtUserinfo() throws Exception {
        mockMvc.perform(get("/userinfo").header(HttpHeaders.AUTHORIZATION, bearer(STUDENT_ID, "STUDENT")))
                .andExpect(status().isUnauthorized());
    }

    // ---- helpers ----

    private record Client(String clientId, String secret, String redirectUri) {
    }

    private Client registerClient() throws Exception {
        String clientId = "module-" + shortId();
        String redirectUri = "https://" + clientId + ".example/auth/callback";

        JsonNode body = json.readTree(mockMvc.perform(post("/api/v1/sso/clients")
                        .header(HttpHeaders.AUTHORIZATION, bearer(ADMIN_ID, "ADMIN"))
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(clientJson(clientId, redirectUri)))
                .andExpect(status().isCreated())
                .andExpect(header().doesNotExist(HttpHeaders.SET_COOKIE))
                .andReturn().getResponse().getContentAsString());

        return new Client(clientId, body.get("clientSecret").asString(), redirectUri);
    }

    private String loginAndGetCode(Client client, Pkce pkce) throws Exception {
        return loginAndGetCode(client, pkce, STUDENT_ID);
    }

    private String loginAndGetCode(Client client, Pkce pkce, long userId) throws Exception {
        Cookie session = mockMvc.perform(authorize(client, pkce))
                .andReturn().getResponse().getCookie(SESSION_COOKIE);
        MockHttpServletResponse started = mockMvc.perform(post("/api/v1/auth/sso-session")
                        .header(HttpHeaders.AUTHORIZATION, bearer(userId, "STUDENT"))
                        .cookie(session))
                .andExpect(status().isOk())
                .andReturn().getResponse();
        String continueUrl = json.readTree(started.getContentAsString()).get("continueUrl").asString();
        String callback = mockMvc.perform(get(URI.create(continueUrl))
                        .cookie(started.getCookie(SESSION_COOKIE))
                        .accept(MediaType.TEXT_HTML))
                .andReturn().getResponse().getRedirectedUrl();
        String code = UriComponentsBuilder.fromUriString(callback).build().getQueryParams().getFirst("code");
        assertThat(code).as("code in " + callback).isNotBlank();
        return code;
    }

    private static MockHttpServletRequestBuilder authorize(Client client, Pkce pkce) {
        return authorize(client, pkce, "openid profile email unicloud");
    }

    private static MockHttpServletRequestBuilder authorize(Client client, Pkce pkce, String scope) {
        return get("/oauth2/authorize")
                .queryParam("response_type", "code")
                .queryParam("client_id", client.clientId())
                .queryParam("redirect_uri", client.redirectUri())
                .queryParam("scope", scope)
                .queryParam("state", STATE)
                .queryParam("nonce", NONCE)
                .queryParam("code_challenge", pkce.challenge())
                .queryParam("code_challenge_method", "S256")
                .accept(MediaType.TEXT_HTML);
    }

    private static MockHttpServletRequestBuilder exchange(Client client, String code, String verifier) {
        return post("/oauth2/token")
                .header(HttpHeaders.AUTHORIZATION, "Basic " + Base64.getEncoder().encodeToString(
                        (client.clientId() + ":" + client.secret()).getBytes(StandardCharsets.UTF_8)))
                .contentType(MediaType.APPLICATION_FORM_URLENCODED)
                .param("grant_type", "authorization_code")
                .param("code", code)
                .param("redirect_uri", client.redirectUri())
                .param("code_verifier", verifier);
    }

    private static String clientJson(String clientId, String redirectUri) {
        return """
                {"clientId": "%s", "name": "Test module", "redirectUris": ["%s"],
                 "postLogoutRedirectUris": ["%s"]}
                """.formatted(clientId, redirectUri, logoutUri(redirectUri));
    }

    private static String logoutUri(String redirectUri) {
        return redirectUri.replace("/auth/callback", "/logged-out");
    }

    private JsonNode payload(String jwt) throws Exception {
        return json.readTree(Base64.getUrlDecoder().decode(jwt.split("\\.")[1]));
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
