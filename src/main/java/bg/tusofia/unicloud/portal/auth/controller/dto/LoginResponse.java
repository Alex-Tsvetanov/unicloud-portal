package bg.tusofia.unicloud.portal.auth.controller.dto;

public record LoginResponse(
        String accessToken,
        String tokenType,
        long expiresIn
) {}
