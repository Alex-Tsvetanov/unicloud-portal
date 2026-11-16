package bg.tusofia.unicloud.portal;

import bg.tusofia.unicloud.portal.auth.application.AccessTokenIssuer;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import bg.tusofia.unicloud.portal.user.domain.User;
import jakarta.servlet.Filter;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.http.MediaType;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;

import java.time.LocalDateTime;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.post;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * The role in a login token must drive hasRole(...) checks, and a failed check must answer 403.
 * Requests go through the real security filter chain with tokens from the real issuer.
 */
@SpringBootTest
class AuthorizationTests {

	/** A user id no row in the test data has, so nothing is ever written. */
	private static final long UNKNOWN_USER_ID = 424242L;

	private static final String STUDENT_REQUEST = """
			{
			  "email": "new.student@student.tu-sofia.bg",
			  "password": "correct horse battery staple",
			  "firstName": "Нов",
			  "lastName": "Студент",
			  "facultyNumber": "121219099",
			  "specialty": "Компютърни науки",
			  "studyYear": 1,
			  "studentStatus": "ACTIVE"
			}
			""";

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
	void meReportsTheRoleFromTheToken() throws Exception {
		mockMvc.perform(get("/api/v1/auth/me")
						.header("Authorization", bearer("STUDENT")))
				.andExpect(status().isOk())
				.andExpect(jsonPath("$.userId").value(String.valueOf(UNKNOWN_USER_ID)))
				.andExpect(jsonPath("$.role").value("STUDENT"));
	}

	@Test
	void roleInTheTokenSatisfiesHasRole() throws Exception {
		// An inspector passes the check on POST /api/v1/students. The request then fails inside the
		// service because the inspector's own user row does not exist, which proves the check passed.
		mockMvc.perform(post("/api/v1/students")
						.header("Authorization", bearer("INSPECTOR"))
						.contentType(MediaType.APPLICATION_JSON)
						.content(STUDENT_REQUEST))
				.andExpect(status().isBadRequest())
				.andExpect(jsonPath("$.message").value("Invalid principal"));
	}

	@Test
	void wrongRoleIsForbiddenNotAServerError() throws Exception {
		mockMvc.perform(post("/api/v1/students")
						.header("Authorization", bearer("STUDENT"))
						.contentType(MediaType.APPLICATION_JSON)
						.content(STUDENT_REQUEST))
				.andExpect(status().isForbidden())
				.andExpect(jsonPath("$.error").value("Forbidden"));
	}

	@Test
	void noTokenIsUnauthorized() throws Exception {
		mockMvc.perform(get("/api/v1/auth/me"))
				.andExpect(status().isUnauthorized());
	}

	private String bearer(String role) {
		LocalDateTime now = LocalDateTime.now();
		User user = User.reconstitute(UNKNOWN_USER_ID, "someone@tu-sofia.bg", "hash", "Test", "User",
				role, null, true, now, now);
		return "Bearer " + tokenIssuer.issue(user).value();
	}

}
