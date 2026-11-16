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
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;
import org.springframework.test.web.servlet.MockMvc;
import org.springframework.test.web.servlet.setup.MockMvcBuilders;
import org.springframework.web.context.WebApplicationContext;

import java.time.LocalDateTime;

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * What the frontend in src/main/resources/static relies on: its pages load without a token, and
 * GET /api/v1/users/me returns the caller's profile. Users come from the V2 test data.
 */
@SpringBootTest
class FrontendTests {

	/** student1@student.tu-sofia.bg in V2__test_data.sql: an active student. */
	private static final long STUDENT_ID = 4L;

	/** ivan.petrov@tu-sofia.bg in V2__test_data.sql: a teacher, so no student record. */
	private static final long TEACHER_ID = 1L;

	/** A user id no row in the test data has. */
	private static final long UNKNOWN_USER_ID = 424242L;

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
	void pagesLoadWithoutAToken() throws Exception {
		for (String path : new String[] {"/login.html", "/profile.html", "/courses.html",
				"/css/app.css", "/js/api.js"}) {
			mockMvc.perform(get(path)).andExpect(status().isOk());
		}
	}

	@Test
	void apiStillNeedsAToken() throws Exception {
		mockMvc.perform(get("/api/v1/users/me"))
				.andExpect(status().isUnauthorized());
	}

	@Test
	void studentSeesTheirAcademicProfile() throws Exception {
		mockMvc.perform(get("/api/v1/users/me")
						.header("Authorization", bearer(STUDENT_ID, "STUDENT")))
				.andExpect(status().isOk())
				.andExpect(jsonPath("$.id").value(4))
				.andExpect(jsonPath("$.email").value("student1@student.tu-sofia.bg"))
				.andExpect(jsonPath("$.firstName").value("Александър"))
				.andExpect(jsonPath("$.role").value("STUDENT"))
				.andExpect(jsonPath("$.student.facultyNumber").value("121219001"))
				.andExpect(jsonPath("$.student.studyYear").value(2))
				.andExpect(jsonPath("$.student.status").value("ACTIVE"));
	}

	@Test
	void staffHaveNoStudentRecord() throws Exception {
		mockMvc.perform(get("/api/v1/users/me")
						.header("Authorization", bearer(TEACHER_ID, "TEACHER")))
				.andExpect(status().isOk())
				.andExpect(jsonPath("$.role").value("TEACHER"))
				.andExpect(jsonPath("$.student").doesNotExist());
	}

	@Test
	void unknownUserIsNotFound() throws Exception {
		mockMvc.perform(get("/api/v1/users/me")
						.header("Authorization", bearer(UNKNOWN_USER_ID, "STUDENT")))
				.andExpect(status().isNotFound());
	}

	private String bearer(long userId, String role) {
		LocalDateTime now = LocalDateTime.now();
		User user = User.reconstitute(userId, "someone@tu-sofia.bg", "hash", "Test", "User",
				role, null, true, now, now);
		return "Bearer " + tokenIssuer.issue(user).value();
	}

}
