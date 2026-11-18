package bg.tusofia.unicloud.portal.staging;

import bg.tusofia.unicloud.portal.auth.application.AccessTokenIssuer;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import bg.tusofia.unicloud.portal.user.domain.User;
import jakarta.servlet.Filter;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.HttpHeaders;
import org.springframework.http.MediaType;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;

import java.time.LocalDateTime;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/** Without the staging profile (production), the staging tools are closed, even to an admin. */
@SpringBootTest
class StagingToolsAbsentTests {

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
    private AccessTokenIssuer tokenIssuer;

    private MockMvc mockMvc;

    @BeforeEach
    void setUp() {
        mockMvc = MockMvcBuilders.webAppContextSetup(context)
                .addFilters(springSecurityFilterChain)
                .build();
    }

    @Test
    void anonymousCallersAreRefused() throws Exception {
        mockMvc.perform(get("/staging")).andExpect(status().isUnauthorized());
        mockMvc.perform(get("/api/v1/staging/users")).andExpect(status().isUnauthorized());
        mockMvc.perform(post("/api/v1/staging/users")
                        .contentType(MediaType.APPLICATION_JSON)
                        .content("{\"email\": \"x@tu-sofia.bg\", \"password\": \"long-enough\", "
                                + "\"firstName\": \"X\", \"lastName\": \"Y\", \"role\": \"ADMIN\"}"))
                .andExpect(status().isUnauthorized());
    }

    @Test
    void evenAnAdminIsRefused() throws Exception {
        LocalDateTime now = LocalDateTime.now();
        User admin = User.reconstitute(9L, "admin@tu-sofia.bg", "hash", "Admin", "User",
                "ADMIN", null, true, now, now);

        mockMvc.perform(get("/api/v1/staging/roles")
                        .header(HttpHeaders.AUTHORIZATION, "Bearer " + tokenIssuer.issue(admin).value()))
                .andExpect(status().isForbidden());
    }
}
