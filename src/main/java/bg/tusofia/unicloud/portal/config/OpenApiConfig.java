package bg.tusofia.unicloud.portal.config;

import io.swagger.v3.oas.models.Components;
import io.swagger.v3.oas.models.OpenAPI;
import io.swagger.v3.oas.models.info.Info;
import io.swagger.v3.oas.models.security.SecurityRequirement;
import io.swagger.v3.oas.models.security.SecurityScheme;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;

/**
 * The API description served at /v3/api-docs and shown by Swagger UI at /swagger-ui.html.
 * Every operation needs the login token unless it says otherwise (see AuthController.login).
 */
@Configuration
public class OpenApiConfig {

    /** The name operations refer to; Swagger UI's "Authorize" button fills it in. */
    public static final String BEARER_JWT = "bearer-jwt";

    @Bean
    OpenAPI portalOpenApi(@Value("${GIT_SHA:dev}") String version) {
        return new OpenAPI()
                .info(new Info()
                        .title("UniCloud Portal API")
                        .version(version)
                        .description("""
                                Accounts, academic profiles and course enrollment: the single \
                                source of truth for who is who across UniCloud.

                                To try a protected endpoint: call `POST /api/v1/auth/login`, copy \
                                `accessToken` from the response, click **Authorize** and paste it."""))
                .components(new Components()
                        .addSecuritySchemes(BEARER_JWT, new SecurityScheme()
                                .type(SecurityScheme.Type.HTTP)
                                .scheme("bearer")
                                .bearerFormat("JWT")
                                .description("The accessToken from POST /api/v1/auth/login")))
                .addSecurityItem(new SecurityRequirement().addList(BEARER_JWT));
    }
}
