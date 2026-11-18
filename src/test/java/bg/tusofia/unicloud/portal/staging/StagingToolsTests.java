package bg.tusofia.unicloud.portal.staging;

import bg.tusofia.unicloud.portal.sso.application.SsoCallbackPolicy;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import jakarta.servlet.Filter;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.MediaType;
import org.springframework.test.context.ActiveProfiles;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;
import tools.jackson.databind.JsonNode;
import tools.jackson.databind.ObjectMapper;

import java.nio.charset.StandardCharsets;
import java.util.Base64;
import java.util.UUID;

import static org.assertj.core.api.Assertions.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.hasItem;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.put;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.content;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * The staging tools (profile "staging"): with no login, a module team makes test users with
 * exactly the permissions it needs, logs in as them, and registers its module for SSO.
 */
@SpringBootTest
@ActiveProfiles("staging")
class StagingToolsTests {

    private static final String PASSWORD = "staging-pass-123";

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
    private SsoCallbackPolicy callbackPolicy;

    private MockMvc mockMvc;

    @BeforeEach
    void setUp() {
        mockMvc = MockMvcBuilders.webAppContextSetup(context)
                .addFilters(springSecurityFilterChain)
                .build();
    }

    @Test
    void aUserWithExactlyThePermissionsAskedForCanLogIn() throws Exception {
        String email = uniqueEmail();
        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(email, "TEACHER", "[\"STUDENT_PROFILE_READ_ANY\", \"COURSE_CREATE_MODIFY\"]", null)))
                .andExpect(status().isCreated())
                .andExpect(jsonPath("$.role").value("TEACHER"))
                .andExpect(jsonPath("$.permissions.length()").value(2));

        JsonNode token = loginToken(email);
        assertThat(token.get("role").asString()).isEqualTo("TEACHER");
        assertThat(token.get("permissions").toString())
                .contains("STUDENT_PROFILE_READ_ANY", "COURSE_CREATE_MODIFY")
                .doesNotContain("COURSE_OFFERING_READ");
    }

    @Test
    void withoutAListAUserGetsTheRoleDefaults() throws Exception {
        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "INSPECTOR", null, null)))
                .andExpect(status().isCreated())
                .andExpect(jsonPath("$.permissions", hasItem("STUDENT_CREATE")))
                .andExpect(jsonPath("$.permissions.length()").value(5));
    }

    @Test
    void anEmptyListMeansNoPermissions() throws Exception {
        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "STUDENT", "[]", null)))
                .andExpect(status().isCreated())
                .andExpect(jsonPath("$.permissions.length()").value(0));
    }

    @Test
    void aStudentCanHaveAnAcademicRecord() throws Exception {
        String facultyNumber = "S" + UUID.randomUUID().toString().substring(0, 8);
        String student = """
                {"facultyNumber": "%s", "specialty": "Компютърни науки", "studyYear": 3, "status": "INTERRUPTED"}
                """.formatted(facultyNumber);

        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "STUDENT", null, student)))
                .andExpect(status().isCreated())
                .andExpect(jsonPath("$.student").value(true));

        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "STUDENT", null, student)))
                .andExpect(status().isConflict());
    }

    @Test
    void permissionsCanBeReplacedLater() throws Exception {
        long id = json.readTree(mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "STUDENT", null, null)))
                .andReturn().getResponse().getContentAsString()).get("id").asLong();

        mockMvc.perform(put("/api/v1/staging/users/" + id + "/permissions")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"permissions\": [\"ENROLLMENT_REQUEST_ACCEPT_REJECT\"]}"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.permissions[0]").value("ENROLLMENT_REQUEST_ACCEPT_REJECT"))
                .andExpect(jsonPath("$.permissions.length()").value(1));

        mockMvc.perform(get("/api/v1/staging/users"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$[?(@.id == " + id + ")].permissions[0]").value("ENROLLMENT_REQUEST_ACCEPT_REJECT"));
    }

    @Test
    void mistakesAreExplained() throws Exception {
        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "STUDENT", "[\"FLY\"]", null)))
                .andExpect(status().isBadRequest())
                .andExpect(jsonPath("$.message", containsString("unknown permission 'FLY'")));

        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson(uniqueEmail(), "DEAN", null, null)))
                .andExpect(status().isBadRequest());

        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content(userJson("student1@student.tu-sofia.bg", "STUDENT", null, null)))
                .andExpect(status().isConflict());
    }

    @Test
    void aModuleRegistersForSsoWithoutAnAdmin() throws Exception {
        mockMvc.perform(post("/api/v1/staging/sso-clients")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("""
                                {"clientId": "module-%s", "name": "Module X",
                                 "redirectUris": ["http://localhost:3000/auth/callback"]}
                                """.formatted(UUID.randomUUID().toString().substring(0, 8))))
                .andExpect(status().isCreated())
                .andExpect(jsonPath("$.clientSecret").isNotEmpty());
    }

    @Test
    void thePageAndTheRoleListAreThere() throws Exception {
        mockMvc.perform(get("/staging"))
                .andExpect(status().isOk())
                .andExpect(content().contentTypeCompatibleWith(MediaType.TEXT_HTML))
                .andExpect(content().string(containsString("Staging tools")));

        mockMvc.perform(get("/api/v1/staging/roles"))
                .andExpect(status().isOk())
                .andExpect(jsonPath("$.roleDefaults.ADMIN[0]").value("STAFF_CREATE"))
                .andExpect(jsonPath("$.permissions.length()").value(10));
    }

    @Test
    void stagingAcceptsAnyCallback() {
        assertThat(callbackPolicy.acceptsAnyCallback()).isTrue();
    }

    private JsonNode loginToken(String email) throws Exception {
        String body = mockMvc.perform(post("/api/v1/auth/login")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"email\": \"%s\", \"password\": \"%s\"}".formatted(email, PASSWORD)))
                .andExpect(status().isOk())
                .andReturn().getResponse().getContentAsString();
        String token = json.readTree(body).get("accessToken").asString();
        return json.readTree(new String(Base64.getUrlDecoder().decode(token.split("\\.")[1]), StandardCharsets.UTF_8));
    }

    private static String userJson(String email, String role, String permissions, String student) {
        return """
                {"email": "%s", "password": "%s", "firstName": "Test", "lastName": "User", "role": "%s"%s%s}
                """.formatted(email, PASSWORD, role,
                permissions == null ? "" : ", \"permissions\": " + permissions,
                student == null ? "" : ", \"student\": " + student);
    }

    private static String uniqueEmail() {
        return "staging-" + UUID.randomUUID().toString().substring(0, 8) + "@test.tu-sofia.bg";
    }
}
