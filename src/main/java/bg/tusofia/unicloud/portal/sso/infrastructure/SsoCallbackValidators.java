package bg.tusofia.unicloud.portal.sso.infrastructure;

import bg.tusofia.unicloud.portal.sso.application.SsoCallbackPolicy;
import org.springframework.security.oauth2.core.OAuth2AuthenticationException;
import org.springframework.security.oauth2.core.OAuth2Error;
import org.springframework.security.oauth2.core.OAuth2ErrorCodes;
import org.springframework.security.oauth2.server.authorization.authentication.OAuth2AuthorizationCodeRequestAuthenticationContext;
import org.springframework.security.oauth2.server.authorization.authentication.OAuth2AuthorizationCodeRequestAuthenticationException;
import org.springframework.security.oauth2.server.authorization.authentication.OAuth2AuthorizationCodeRequestAuthenticationToken;
import org.springframework.security.oauth2.server.authorization.authentication.OAuth2AuthorizationCodeRequestAuthenticationValidator;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClient;
import org.springframework.security.oauth2.server.authorization.oidc.authentication.OidcLogoutAuthenticationContext;
import org.springframework.security.oauth2.server.authorization.oidc.authentication.OidcLogoutAuthenticationToken;
import org.springframework.security.oauth2.server.authorization.oidc.authentication.OidcLogoutAuthenticationValidator;

import java.util.Set;
import java.util.function.Consumer;

/**
 * The library's checks of /oauth2/authorize and /connect/logout requests, with the callback rule
 * of {@link SsoCallbackPolicy} applied on top.
 *
 * <p>{@code any}: the library demands that a callback is registered for the module. To accept
 * any callback without copying the library's other checks (PKCE, scopes, prompt), the request's
 * callback is added to a copy of the module's registration for the check only. Nothing is saved.
 *
 * <p>{@code allowlist}: the library's checks run unchanged, then the callback's host must still
 * be allowed. A refused /oauth2/authorize answers 400 without a redirect.
 */
public final class SsoCallbackValidators {

    private SsoCallbackValidators() {
    }

    public static Consumer<OAuth2AuthorizationCodeRequestAuthenticationContext> authorization(
            SsoCallbackPolicy policy) {

        Consumer<OAuth2AuthorizationCodeRequestAuthenticationContext> library =
                new OAuth2AuthorizationCodeRequestAuthenticationValidator();

        return context -> {
            OAuth2AuthorizationCodeRequestAuthenticationToken request = context.getAuthentication();
            String redirectUri = request.getRedirectUri();

            if (policy.acceptsAnyCallback()) {
                library.accept(redirectUri != null && policy.permits(redirectUri)
                        ? withCallback(context, redirectUri)
                        : context);
                return;
            }

            library.accept(context);
            String used = redirectUri != null ? redirectUri : onlyRegistered(context.getRegisteredClient().getRedirectUris());
            if (used != null && !policy.permits(used)) {
                throw new OAuth2AuthorizationCodeRequestAuthenticationException(
                        invalidRequest("redirect_uri"), null);
            }
        };
    }

    public static Consumer<OidcLogoutAuthenticationContext> logout(SsoCallbackPolicy policy) {
        Consumer<OidcLogoutAuthenticationContext> library = new OidcLogoutAuthenticationValidator();

        return context -> {
            OidcLogoutAuthenticationToken request = context.getAuthentication();
            String logoutUri = request.getPostLogoutRedirectUri();

            if (policy.acceptsAnyCallback() && logoutUri != null && policy.permits(logoutUri)) {
                RegisteredClient widened = RegisteredClient.from(context.getRegisteredClient())
                        .postLogoutRedirectUri(logoutUri)
                        .build();
                library.accept(OidcLogoutAuthenticationContext.with(request).registeredClient(widened).build());
                return;
            }

            library.accept(context);
            if (!policy.acceptsAnyCallback() && logoutUri != null && !policy.permits(logoutUri)) {
                throw new OAuth2AuthenticationException(invalidRequest("post_logout_redirect_uri"));
            }
        };
    }

    private static OAuth2AuthorizationCodeRequestAuthenticationContext withCallback(
            OAuth2AuthorizationCodeRequestAuthenticationContext context, String redirectUri) {

        RegisteredClient widened = RegisteredClient.from(context.getRegisteredClient())
                .redirectUri(redirectUri)
                .build();
        OAuth2AuthorizationCodeRequestAuthenticationContext.Builder copy =
                OAuth2AuthorizationCodeRequestAuthenticationContext.with(context.getAuthentication())
                        .registeredClient(widened);
        if (context.getAuthorizationRequest() != null) {
            copy.authorizationRequest(context.getAuthorizationRequest());
        }
        if (context.getAuthorizationConsent() != null) {
            copy.authorizationConsent(context.getAuthorizationConsent());
        }
        return copy.build();
    }

    private static String onlyRegistered(Set<String> uris) {
        return uris.size() == 1 ? uris.iterator().next() : null;
    }

    private static OAuth2Error invalidRequest(String parameter) {
        return new OAuth2Error(OAuth2ErrorCodes.INVALID_REQUEST, "OAuth 2.0 Parameter: " + parameter, null);
    }
}
