package bg.tusofia.unicloud.portal.sso.infrastructure;

import org.springframework.jdbc.core.JdbcOperations;
import org.springframework.security.oauth2.core.OAuth2AccessToken;
import org.springframework.security.oauth2.core.OAuth2AuthenticationException;
import org.springframework.security.oauth2.core.OAuth2Error;
import org.springframework.security.oauth2.core.OAuth2ErrorCodes;
import org.springframework.security.oauth2.core.oidc.OidcIdToken;
import org.springframework.security.oauth2.server.authorization.JdbcOAuth2AuthorizationService;
import org.springframework.security.oauth2.server.authorization.OAuth2Authorization;
import org.springframework.security.oauth2.server.authorization.OAuth2AuthorizationCode;
import org.springframework.security.oauth2.server.authorization.client.RegisteredClientRepository;

import java.util.Objects;

/**
 * The library's JDBC store, plus a guarantee it does not give: an authorization code is
 * exchanged for tokens at most once, even when two replicas get the same code at the same moment.
 *
 * <p>The library checks "not used yet", issues the tokens, then saves the code as used. Two
 * replicas can both pass the check before either saves. Here the save that attaches new tokens
 * to a used code first flips {@code code_redeemed} with a conditional UPDATE. The database lets
 * exactly one such UPDATE change the row; every other exchange fails with {@code invalid_grant}
 * before its tokens leave the server.
 */
public class SingleUseCodeAuthorizationService extends JdbcOAuth2AuthorizationService {

    private static final String CLAIM_CODE =
            "UPDATE oauth2_authorization SET code_redeemed = TRUE WHERE id = ? AND code_redeemed = FALSE";

    public SingleUseCodeAuthorizationService(
            JdbcOperations jdbcOperations,
            RegisteredClientRepository registeredClientRepository) {

        super(jdbcOperations, registeredClientRepository);
    }

    @Override
    public void save(OAuth2Authorization authorization) {
        if (redeemsCode(authorization)
                && getJdbcOperations().update(CLAIM_CODE, authorization.getId()) != 1) {
            revokeIssuedTokens(authorization.getId());
            throw new OAuth2AuthenticationException(new OAuth2Error(
                    OAuth2ErrorCodes.INVALID_GRANT,
                    "The authorization code has already been used",
                    null));
        }
        super.save(authorization);
    }

    /**
     * The code was used twice, so one of the two callers was not the module: the tokens the
     * other exchange received are revoked too, as the library does for a replay that comes later.
     */
    private void revokeIssuedTokens(String authorizationId) {
        OAuth2Authorization stored = findById(authorizationId);
        if (stored == null) {
            return;
        }
        OAuth2Authorization.Builder revoked = OAuth2Authorization.from(stored);
        if (stored.getAccessToken() != null) {
            revoked.invalidate(stored.getAccessToken().getToken());
        }
        OAuth2Authorization.Token<OidcIdToken> idToken = stored.getToken(OidcIdToken.class);
        if (idToken != null) {
            revoked.invalidate(idToken.getToken());
        }
        super.save(revoked.build());
    }

    /**
     * True for the save that hands out new tokens for a used code: the code is marked used and
     * the access token differs from the one stored (none, or another exchange's). Later saves of
     * the same authorization, such as a logout, keep the same access token and pass through.
     */
    private boolean redeemsCode(OAuth2Authorization authorization) {
        OAuth2Authorization.Token<OAuth2AuthorizationCode> code =
                authorization.getToken(OAuth2AuthorizationCode.class);
        OAuth2Authorization.Token<OAuth2AccessToken> accessToken = authorization.getAccessToken();

        if (code == null || !code.isInvalidated() || accessToken == null) {
            return false;
        }

        OAuth2Authorization stored = findById(authorization.getId());
        String storedValue = stored == null || stored.getAccessToken() == null
                ? null
                : stored.getAccessToken().getToken().getTokenValue();

        return !Objects.equals(storedValue, accessToken.getToken().getTokenValue());
    }
}
