package bg.tusofia.unicloud.portal.auth.application;

public record AccessToken(
        String value,
        long expiresInSeconds
) {}
