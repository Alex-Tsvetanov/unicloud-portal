package bg.tusofia.unicloud.portal.sso.controller.dto;

/**
 * {@code continueUrl}: the /oauth2/authorize path and query the login page must open next, or
 * null when no module login is waiting (the session expired, or the user came straight to the
 * portal).
 */
public record SsoSessionResponse(String continueUrl) {
}
