package bg.tusofia.unicloud.portal.user.service;

public record RegisterUserCommand(
        String email,
        String password,
        String firstName,
        String lastName,
        String role
) {}
