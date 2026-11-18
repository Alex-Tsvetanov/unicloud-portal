package bg.tusofia.unicloud.portal.sso.application;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import java.net.URI;
import java.net.URISyntaxException;
import java.util.List;
import java.util.Locale;
import java.util.Set;

/**
 * Which callback URLs (redirect_uri, post_logout_redirect_uri) the portal accepts
 * (docs/sso.md, "Callback URLs per environment").
 *
 * <ul>
 *   <li>{@code allowlist} (the default, production): a callback must be registered for the
 *       module and match exactly, and its host must be listed in {@code app.sso.allowed-callback-hosts}
 *       (the file sso-callback-hosts.properties). https only; plain http only for a listed
 *       loopback host.</li>
 *   <li>{@code any} (staging): any absolute http or https URL, registered or
 *       not, so module developers can point a test login anywhere.</li>
 * </ul>
 */
@Component
public class SsoCallbackPolicy {

    public enum Mode { ALLOWLIST, ANY }

    private static final Set<String> LOOPBACK_HOSTS = Set.of("localhost", "127.0.0.1", "[::1]");

    private final Mode mode;
    private final List<String> allowedHosts;

    public SsoCallbackPolicy(
            @Value("${app.sso.callback-policy:allowlist}") String mode,
            @Value("${app.sso.allowed-callback-hosts:}") List<String> allowedHosts) {

        this.mode = Mode.valueOf(mode.strip().toUpperCase(Locale.ROOT));
        this.allowedHosts = allowedHosts.stream()
                .map(host -> host.strip().toLowerCase(Locale.ROOT))
                .filter(host -> !host.isEmpty())
                .toList();
    }

    public boolean acceptsAnyCallback() {
        return mode == Mode.ANY;
    }

    public List<String> allowedHosts() {
        return allowedHosts;
    }

    /**
     * Throws {@link IllegalArgumentException} with a message for the admin when {@code value} may
     * not be registered as a callback.
     */
    public void checkRegistration(String value, String field) {
        URI uri = parse(value, field);
        if (!isAcceptable(uri)) {
            throw new IllegalArgumentException(field + ": " + rule() + ": " + value);
        }
        if (uri.getRawFragment() != null || uri.getRawUserInfo() != null || value.contains("*")
                || value.contains(",")) {
            throw new IllegalArgumentException(
                    field + ": must not contain a fragment, user info, '*' or ',': " + value);
        }
    }

    /**
     * True when a login may use this callback, on top of the library's own rules. In
     * {@code allowlist} mode the library already demands an exact match with a registered URL;
     * this adds the host check, so taking a host out of the file switches its callbacks off.
     */
    public boolean permits(String value) {
        try {
            return isAcceptable(new URI(value));
        } catch (URISyntaxException e) {
            return false;
        }
    }

    private boolean isAcceptable(URI uri) {
        String scheme = uri.getScheme() == null ? "" : uri.getScheme().toLowerCase(Locale.ROOT);
        String host = uri.getHost() == null ? "" : uri.getHost().toLowerCase(Locale.ROOT);
        if (!uri.isAbsolute() || host.isEmpty() || !(scheme.equals("https") || scheme.equals("http"))) {
            return false;
        }
        if (mode == Mode.ANY) {
            return true;
        }
        boolean secure = scheme.equals("https") || LOOPBACK_HOSTS.contains(host);
        return secure && isAllowedHost(host);
    }

    /** Exact host, or "*.example.org" for any subdomain of example.org (not example.org itself). */
    private boolean isAllowedHost(String host) {
        for (String allowed : allowedHosts) {
            if (allowed.startsWith("*.") ? host.endsWith(allowed.substring(1)) : host.equals(allowed)) {
                return true;
            }
        }
        return false;
    }

    private String rule() {
        return mode == Mode.ANY
                ? "must be an absolute http or https URL"
                : "must be an https URL on a host listed in sso-callback-hosts.properties "
                        + "(http only for a listed localhost), allowed hosts: " + allowedHosts;
    }

    private static URI parse(String value, String field) {
        try {
            return new URI(value);
        } catch (URISyntaxException e) {
            throw new IllegalArgumentException(field + ": not a valid URL: " + value);
        }
    }
}
