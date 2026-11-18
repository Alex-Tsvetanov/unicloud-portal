package bg.tusofia.unicloud.portal.staging.dto;

import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotNull;

import java.util.List;

/** The user's new, complete set of permissions, by name. An empty list removes them all. */
public record SetPermissionsRequest(@NotNull List<@NotBlank String> permissions) {
}
