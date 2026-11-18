package bg.tusofia.unicloud.portal.sso.controller.dto;

import java.util.List;

/** The new client. {@code clientSecret} appears here once; the portal keeps only its hash. */
public record RegisterSsoClientResponse(
        String clientId,
        String clientSecret,
        List<String> redirectUris,
        List<String> postLogoutRedirectUris
) {
}
