package bg.tusofia.unicloud.portal;

import bg.tusofia.unicloud.portal.auth.application.AccessToken;
import bg.tusofia.unicloud.portal.auth.application.AccessTokenIssuer;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import bg.tusofia.unicloud.portal.user.domain.User;
import org.junit.jupiter.api.Test;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.security.oauth2.jwt.Jwt;
import org.springframework.security.oauth2.jwt.JwtDecoder;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;

import java.time.LocalDateTime;

import static org.assertj.core.api.Assertions.assertThat;

@SpringBootTest
class PortalApplicationTests {

	@DynamicPropertySource
	static void jwtKeys(DynamicPropertyRegistry registry) {
		TestJwtKeys.register(registry);
	}

	@Autowired
	private AccessTokenIssuer tokenIssuer;

	@Autowired
	private JwtDecoder jwtDecoder;

	@Test
	void contextLoads() {
	}

	@Test
	void issuedTokenIsAcceptedByTheDecoder() {
		LocalDateTime now = LocalDateTime.now();
		User user = User.reconstitute(42L, "student@tu-sofia.bg", "hash", "Test", "Student",
				"STUDENT", null, true, now, now);

		AccessToken token = tokenIssuer.issue(user);
		Jwt jwt = jwtDecoder.decode(token.value());

		assertThat(jwt.getSubject()).isEqualTo("42");
		assertThat(jwt.getClaimAsString("role")).isEqualTo("STUDENT");
		assertThat(jwt.getAudience()).contains("unicloud-api");
	}

}
