package bg.tusofia.unicloud.portal.staging.dto;

import java.util.List;

/** A user as the staging tools show it. Never the password. */
public record StagingUserResponse(
        Long id,
        String email,
        String firstName,
        String lastName,
        String role,
        List<String> permissions,
        boolean student
) {
}
