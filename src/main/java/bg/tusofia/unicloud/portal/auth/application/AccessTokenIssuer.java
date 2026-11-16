package bg.tusofia.unicloud.portal.auth.application;

import bg.tusofia.unicloud.portal.user.domain.User;

public interface AccessTokenIssuer {
    AccessToken issue(User user);
}
