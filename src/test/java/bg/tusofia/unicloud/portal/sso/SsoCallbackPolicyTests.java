package bg.tusofia.unicloud.portal.sso;

import bg.tusofia.unicloud.portal.sso.application.SsoCallbackPolicy;
import org.junit.jupiter.api.Test;

import java.util.List;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatCode;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

/** Which callback URLs each environment accepts (docs/sso.md, "Callback URLs per environment"). */
class SsoCallbackPolicyTests {

    private static SsoCallbackPolicy allowlist(String... hosts) {
        return new SsoCallbackPolicy("allowlist", List.of(hosts));
    }

    @Test
    void productionAcceptsOnlyListedHosts() {
        SsoCallbackPolicy policy = allowlist("library.tu-sofia.bg");

        assertThat(policy.permits("https://library.tu-sofia.bg/auth/callback")).isTrue();
        assertThat(policy.permits("https://LIBRARY.tu-sofia.bg/auth/callback")).as("hosts ignore case").isTrue();
        assertThat(policy.permits("https://evil.example/auth/callback")).isFalse();
        assertThat(policy.permits("https://library.tu-sofia.bg.evil.example/cb")).isFalse();
    }

    @Test
    void wildcardCoversSubdomainsButNotTheDomainItself() {
        SsoCallbackPolicy policy = allowlist("*.tu-sofia.bg");

        assertThat(policy.permits("https://library.tu-sofia.bg/cb")).isTrue();
        assertThat(policy.permits("https://a.b.tu-sofia.bg/cb")).isTrue();
        assertThat(policy.permits("https://tu-sofia.bg/cb")).isFalse();
        assertThat(policy.permits("https://evil-tu-sofia.bg/cb")).isFalse();
    }

    @Test
    void productionNeedsHttpsExceptOnAListedLocalhost() {
        assertThat(allowlist("library.tu-sofia.bg").permits("http://library.tu-sofia.bg/cb")).isFalse();
        assertThat(allowlist("localhost").permits("http://localhost:3000/cb")).isTrue();
        assertThat(allowlist("library.tu-sofia.bg").permits("http://localhost:3000/cb"))
                .as("localhost only when listed").isFalse();
    }

    @Test
    void anEmptyListAcceptsNothing() {
        SsoCallbackPolicy policy = allowlist();

        assertThat(policy.permits("https://library.tu-sofia.bg/cb")).isFalse();
        assertThatThrownBy(() -> policy.checkRegistration("https://library.tu-sofia.bg/cb", "redirectUris"))
                .isInstanceOf(IllegalArgumentException.class)
                .hasMessageContaining("sso-callback-hosts.properties");
    }

    @Test
    void stagingAcceptsAnyHttpOrHttpsUrl() {
        SsoCallbackPolicy policy = new SsoCallbackPolicy("any", List.of());

        assertThat(policy.acceptsAnyCallback()).isTrue();
        assertThat(policy.permits("http://dev-box:3000/auth/callback")).isTrue();
        assertThat(policy.permits("https://anything.example/cb")).isTrue();
        assertThatCode(() -> policy.checkRegistration("http://192.168.1.20:5173/cb", "redirectUris"))
                .doesNotThrowAnyException();
    }

    @Test
    void evenStagingRefusesWhatIsNotAWebUrl() {
        SsoCallbackPolicy policy = new SsoCallbackPolicy("any", List.of());

        assertThat(policy.permits("javascript:alert(1)")).isFalse();
        assertThat(policy.permits("/relative/callback")).isFalse();
        assertThat(policy.permits("ftp://files.example/cb")).isFalse();
    }

    @Test
    void anUnknownPolicyStopsTheApp() {
        assertThatThrownBy(() -> new SsoCallbackPolicy("open", List.of()))
                .isInstanceOf(IllegalArgumentException.class);
    }
}
