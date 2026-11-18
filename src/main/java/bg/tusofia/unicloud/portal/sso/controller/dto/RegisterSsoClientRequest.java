package bg.tusofia.unicloud.portal.sso.controller.dto;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.Pattern;
import jakarta.validation.constraints.Size;

import java.util.List;

/**
 * A module to register for SSO.
 *
 * @param clientId               lower-case letters, digits and dashes, e.g. "library"
 * @param name                   shown in logs and to admins
 * @param redirectUris           every callback URL the module will use, written out in full
 * @param postLogoutRedirectUris where the module may send the user after /connect/logout; may be empty
 */
public record RegisterSsoClientRequest(
        @NotBlank
        @Pattern(regexp = "[a-z0-9][a-z0-9-]{1,62}",
                message = "must be 2 to 63 lower-case letters, digits or dashes")
        String clientId,

        @NotBlank
        @Size(max = 200)
        String name,

        @NotEmpty
        List<@NotBlank String> redirectUris,

        List<@NotBlank String> postLogoutRedirectUris
) {
}
