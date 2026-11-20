//! Settings, all from environment variables. See the design's "Configuration" table.

use anyhow::{Context, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Environment {
    Production,
    Staging,
    Development,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallbackPolicyMode {
    /// Production: registered URLs only, exact match, https, host on the allowed list
    Allowlist,
    /// Staging: any absolute http(s) URL, registered or not
    Any,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub env: Environment,
    pub bind: String,
    pub database_url: String,
    pub db_pool_max: u32,
    /// PostgreSQL schema for every table. Staging uses `portal` to start fresh next to the Java tables.
    pub db_schema: String,
    /// The public origin, used as `iss`. Empty: worked out from each request (development only).
    pub issuer: Option<String>,
    pub signing_key_pem: String,
    pub allowed_callback_hosts: Vec<String>,
    pub callback_policy: CallbackPolicyMode,
    /// `__Host-UNICLOUD_SSO` with `Secure`. Only local tests over plain http turn this off.
    pub secure_cookie: bool,
    pub git_sha: String,
    pub replica: String,
    pub current_semester: String,
    pub demo_password: Option<String>,
    pub bootstrap_admin: Option<(String, String)>,
}

/// The production host list shipped with the image (docs/sso.md, "Callback URLs per environment")
const DEFAULT_CALLBACK_HOSTS: &str = include_str!("../config/sso-callback-hosts.txt");

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let var = |k: &str| std::env::var(k).ok().filter(|v| !v.trim().is_empty());
        let env = match var("APP_ENV").as_deref().map(str::to_ascii_lowercase).as_deref() {
            Some("production") | None => Environment::Production,
            Some("staging") => Environment::Staging,
            Some("development") => Environment::Development,
            Some(other) => bail!("APP_ENV must be production, staging or development, not {other}"),
        };
        // APP_JWT_PRIVATE_KEY: the name the Java portal used for the same key (existing Render settings)
        let signing_key_pem = var("APP_SIGNING_KEY_PEM")
            .or_else(|| var("APP_JWT_PRIVATE_KEY"))
            .context("APP_SIGNING_KEY_PEM is required: the RSA private key (PKCS#8 PEM) that signs tokens")?
            .replace("\\n", "\n");
        let hosts_raw = var("APP_SSO_ALLOWED_CALLBACK_HOSTS").unwrap_or_else(|| parse_hosts_file(DEFAULT_CALLBACK_HOSTS));
        // Render sets RENDER_EXTERNAL_URL to the service's own https address
        let issuer = var("APP_SSO_ISSUER")
            .or_else(|| var("RENDER_EXTERNAL_URL"))
            .map(|s| s.trim().trim_end_matches('/').to_string());
        if env == Environment::Production && !issuer.as_deref().is_some_and(|i| i.starts_with("https://")) {
            bail!("APP_SSO_ISSUER must be the portal's public https address in production");
        }
        let bootstrap_admin = match (var("APP_BOOTSTRAP_ADMIN_EMAIL"), var("APP_BOOTSTRAP_ADMIN_PASSWORD")) {
            (Some(e), Some(p)) => Some((e, p)),
            _ => None,
        };
        Ok(Self {
            env,
            bind: format!("0.0.0.0:{}", var("PORT").unwrap_or_else(|| "8080".into())),
            database_url: var("DATABASE_URL").context("DATABASE_URL is required")?,
            db_pool_max: var("APP_DB_POOL_MAX").map(|v| v.parse()).transpose()?.unwrap_or(10),
            db_schema: {
                let s = var("APP_DB_SCHEMA").unwrap_or_else(|| "public".into());
                if s.is_empty() || !s.chars().all(|c| c.is_ascii_lowercase() || c == '_') {
                    bail!("APP_DB_SCHEMA must be lower-case letters and underscores");
                }
                s
            },
            issuer,
            signing_key_pem,
            allowed_callback_hosts: split_hosts(&hosts_raw),
            callback_policy: if env == Environment::Production {
                CallbackPolicyMode::Allowlist
            } else {
                CallbackPolicyMode::Any
            },
            secure_cookie: env == Environment::Production || var("APP_INSECURE_COOKIE").is_none(),
            git_sha: var("GIT_SHA")
                .filter(|s| s != "dev")
                .or_else(|| var("RENDER_GIT_COMMIT"))
                .unwrap_or_else(|| "dev".into()),
            replica: var("REPLICA_NAME")
                .or_else(|| var("RENDER_INSTANCE_ID"))
                .or_else(|| var("CONTAINER_APP_REPLICA_NAME"))
                .or_else(|| var("HOSTNAME"))
                .unwrap_or_else(|| "local".into()),
            current_semester: var("APP_CURRENT_SEMESTER").unwrap_or_else(|| "2026-27-W".into()),
            demo_password: var("APP_DEMO_PASSWORD"),
            bootstrap_admin,
        })
    }

    pub fn staging_tools(&self) -> bool {
        self.env != Environment::Production
    }

    pub fn cookie_name(&self) -> &'static str {
        if self.secure_cookie {
            "__Host-UNICLOUD_SSO"
        } else {
            "UNICLOUD_SSO"
        }
    }
}

fn parse_hosts_file(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .flat_map(|l| l.split(','))
        .collect::<Vec<_>>()
        .join(",")
}

pub fn split_hosts(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(|h| h.trim().to_ascii_lowercase())
        .filter(|h| !h.is_empty())
        .collect()
}
