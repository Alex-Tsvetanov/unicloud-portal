//! Which callback and logout URLs the portal trusts (docs/sso.md, "Callback URLs per environment").

use url::Url;

use crate::config::CallbackPolicyMode;

const LOOPBACK: [&str; 3] = ["localhost", "127.0.0.1", "[::1]"];

/// The host as it appears in the URL (IPv6 in brackets), lower case
fn host_of(u: &Url) -> Option<String> {
    match u.host()? {
        url::Host::Ipv6(a) => Some(format!("[{a}]")),
        h => Some(h.to_string().to_ascii_lowercase()),
    }
}

/// `library.tu-sofia.bg` matches itself; `*.tu-sofia.bg` matches any subdomain, at any depth, but not `tu-sofia.bg`
pub fn host_allowed(hosts: &[String], host: &str) -> bool {
    let host = host.to_ascii_lowercase();
    hosts.iter().any(|entry| match entry.strip_prefix("*.") {
        Some(domain) => host.ends_with(&format!(".{domain}")),
        None => *entry == host,
    })
}

/// The rules every callback URL must follow in the given mode. Err holds the broken rule.
pub fn check_url(mode: CallbackPolicyMode, hosts: &[String], raw: &str) -> Result<(), String> {
    let u = Url::parse(raw).map_err(|_| "not a valid absolute URL".to_string())?;
    if u.scheme() != "http" && u.scheme() != "https" {
        return Err("must be an http or https URL".into());
    }
    let host = host_of(&u)
        .filter(|h| !h.is_empty())
        .ok_or_else(|| "must have a host".to_string())?;
    if mode == CallbackPolicyMode::Allowlist {
        let scheme_ok = u.scheme() == "https" || LOOPBACK.contains(&host.as_str());
        if !scheme_ok || !host_allowed(hosts, &host) {
            return Err(format!(
                "must be an https URL on a host listed in sso-callback-hosts.txt (http only for a listed localhost), allowed hosts: {hosts:?}"
            ));
        }
    }
    if u.fragment().is_some() || raw.contains('#') {
        return Err("must not have a fragment".into());
    }
    if !u.username().is_empty() || u.password().is_some() {
        return Err("must not contain user info".into());
    }
    if raw.contains('*') || raw.contains(',') {
        return Err("must not contain '*' or ','".into());
    }
    Ok(())
}

/// Is this URL trusted for this module right now? Allowlist: registered exactly and on an allowed host.
/// Any: every absolute http(s) URL that passes the general rules.
pub fn trusted(mode: CallbackPolicyMode, hosts: &[String], registered: &[String], raw: &str) -> bool {
    match mode {
        CallbackPolicyMode::Allowlist => registered.iter().any(|r| r == raw) && check_url(mode, hosts, raw).is_ok(),
        CallbackPolicyMode::Any => check_url(mode, hosts, raw).is_ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use CallbackPolicyMode::*;

    fn hosts() -> Vec<String> {
        vec!["library.tu-sofia.bg".into(), "*.tu-sofia.bg".into(), "localhost".into()]
    }

    #[test]
    fn host_matching_follows_the_java_rules() {
        let h = hosts();
        assert!(host_allowed(&h, "library.tu-sofia.bg"));
        assert!(host_allowed(&h, "LIBRARY.TU-SOFIA.BG"));
        assert!(host_allowed(&h, "a.b.tu-sofia.bg"));
        assert!(!host_allowed(&["*.tu-sofia.bg".to_string()], "tu-sofia.bg"));
        assert!(!host_allowed(&h, "evil-tu-sofia.bg"));
        assert!(!host_allowed(&h, "library.tu-sofia.bg.evil.example"));
    }

    #[test]
    fn allowlist_needs_https_except_loopback() {
        let h = hosts();
        assert!(check_url(Allowlist, &h, "https://library.tu-sofia.bg/auth/callback").is_ok());
        assert!(check_url(Allowlist, &h, "http://library.tu-sofia.bg/cb").is_err());
        assert!(check_url(Allowlist, &h, "http://localhost:3000/cb").is_ok());
        assert!(check_url(Allowlist, &h, "https://evil.example/cb").is_err());
        assert!(check_url(Allowlist, &h, "https://library.tu-sofia.bg:8443/cb").is_ok());
    }

    #[test]
    fn non_web_and_odd_urls_are_refused_everywhere() {
        for mode in [Allowlist, Any] {
            for bad in [
                "javascript:alert(1)",
                "/relative",
                "ftp://library.tu-sofia.bg/x",
                "https://u:p@library.tu-sofia.bg/cb",
                "https://library.tu-sofia.bg/cb#frag",
                "https://library.tu-sofia.bg/*",
            ] {
                assert!(check_url(mode, &hosts(), bad).is_err(), "{bad} in {mode:?}");
            }
        }
    }

    #[test]
    fn exact_match_in_allowlist() {
        let reg = vec!["https://library.tu-sofia.bg/auth/callback".to_string()];
        assert!(trusted(Allowlist, &hosts(), &reg, "https://library.tu-sofia.bg/auth/callback"));
        assert!(!trusted(
            Allowlist,
            &hosts(),
            &reg,
            "https://library.tu-sofia.bg/auth/callback/../admin"
        ));
        assert!(!trusted(Allowlist, &hosts(), &reg, "https://library.tu-sofia.bg/auth/callbackx"));
        assert!(trusted(Any, &[], &reg, "http://localhost:3000/anything"));
    }

    #[test]
    fn removing_a_host_switches_off_registered_callbacks() {
        let reg = vec!["https://library.tu-sofia.bg/auth/callback".to_string()];
        assert!(!trusted(Allowlist, &["other.example".to_string()], &reg, &reg[0]));
    }
}
