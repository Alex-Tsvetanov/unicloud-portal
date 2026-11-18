package bg.tusofia.unicloud.portal;

import bg.tusofia.unicloud.portal.support.TestJwtKeys;
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

import static org.springframework.test.web.servlet.request.MockMvcRequestBuilders.get;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.content;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.jsonPath;
import static org.springframework.test.web.servlet.result.MockMvcResultMatchers.status;

/**
 * The OpenAPI description and Swagger UI must open without a token, describe the /api/**
 * endpoints, and tell clients which ones need the login token.
 */
@SpringBootTest
class ApiDocsTests {

	@DynamicPropertySource
	static void jwtKeys(DynamicPropertyRegistry registry) {
		TestJwtKeys.register(registry);
	}

	@Autowired
	private WebApplicationContext context;

	@Autowired
	@Qualifier("springSecurityFilterChain")
	private Filter springSecurityFilterChain;

	private MockMvc mockMvc;

	@BeforeEach
	void setUp() {
		mockMvc = MockMvcBuilders.webAppContextSetup(context)
				.addFilters(springSecurityFilterChain)
				.build();
	}

	@Test
	void apiDocsOpenWithoutATokenAndDescribeTheApi() throws Exception {
		mockMvc.perform(get("/v3/api-docs"))
				.andExpect(status().isOk())
				.andExpect(jsonPath("$.info.title").value("UniCloud Portal API"))
				.andExpect(jsonPath("$.paths['/api/v1/courses']").exists())
				.andExpect(jsonPath("$.paths['/api/v1/enrollments']").exists())
				// Only the API: the pipeline's endpoints stay out of the docs
				.andExpect(jsonPath("$.paths['/health']").doesNotExist());
	}

	@Test
	void apiDocsSayWhichEndpointsNeedTheToken() throws Exception {
		mockMvc.perform(get("/v3/api-docs"))
				.andExpect(jsonPath("$.components.securitySchemes.bearer-jwt.scheme").value("bearer"))
				// Every operation needs the token by default...
				.andExpect(jsonPath("$.security[0].bearer-jwt").isArray())
				// ...except logging in
				.andExpect(jsonPath("$.paths['/api/v1/auth/login'].post.security").isArray())
				.andExpect(jsonPath("$.paths['/api/v1/auth/login'].post.security").isEmpty());
	}

	@Test
	void swaggerUiOpensWithoutAToken() throws Exception {
		mockMvc.perform(get("/swagger-ui.html"))
				.andExpect(status().is3xxRedirection());
		mockMvc.perform(get("/swagger-ui/index.html"))
				.andExpect(status().isOk())
				.andExpect(content().string(org.hamcrest.Matchers.containsString("swagger-ui")));
	}

	@Test
	void theApiItselfStillNeedsAToken() throws Exception {
		mockMvc.perform(get("/api/v1/courses"))
				.andExpect(status().isUnauthorized());
	}
}
