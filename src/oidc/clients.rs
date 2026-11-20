//! Modules registered for single sign-on: storage, the registration rules, and the admin page.

use axum::Form;
use axum::extract::State;
use maud::{Markup, html};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use super::policy;
use crate::access::Role;
use crate::config::Config;
use crate::crypto::{ct_eq, random_token, sha256_hex};
use crate::error::{AppError, AppResult, is_unique_violation};
use crate::state::AppState;
use crate::ui::components::{csrf_field, empty, field, flash, layout, page_title};
use crate::web::session::CurrentUser;

pub const RESERVED_CLIENT_ID: &str = "unicloud-api";
const MAX_URIS_CHARS: usize = 1000;

#[derive(Debug, Clone, FromRow)]
pub struct Client {
    pub client_id: String,
    pub name: String,
    pub secret_hash: String,
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub directory_access: bool,
}

impl Client {
    pub fn secret_matches(&self, secret: &str) -> bool {
        ct_eq(&sha256_hex(secret), &self.secret_hash)
    }
}

pub async fn find(db: &PgPool, client_id: &str) -> sqlx::Result<Option<Client>> {
    sqlx::query_as(
        "SELECT client_id, name, secret_hash, redirect_uris, post_logout_redirect_uris, directory_access FROM oauth_clients WHERE client_id = $1",
    )
    .bind(client_id)
    .fetch_optional(db)
    .await
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Registration {
    pub client_id: Option<String>,
    pub name: Option<String>,
    pub redirect_uris: Option<Vec<String>>,
    pub post_logout_redirect_uris: Option<Vec<String>>,
    #[serde(default)]
    pub directory: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Registered {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uris: Vec<String>,
    pub post_logout_redirect_uris: Vec<String>,
    pub directory: bool,
}

#[derive(Debug)]
pub enum RegisterError {
    /// Bean-validation style: field and message
    Fields(Vec<(&'static str, String)>),
    Invalid(String),
    Duplicate(String),
    Db(sqlx::Error),
}

pub fn valid_client_id(id: &str) -> bool {
    let b = id.as_bytes();
    (2..=63).contains(&b.len())
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-')
}

fn clean_uris(field: &str, list: Vec<String>, cfg: &Config) -> Result<Vec<String>, RegisterError> {
    let mut out: Vec<String> = Vec::new();
    for u in list.into_iter().map(|u| u.trim().to_string()) {
        if !out.contains(&u) {
            out.push(u);
        }
    }
    for u in &out {
        policy::check_url(cfg.callback_policy, &cfg.allowed_callback_hosts, u)
            .map_err(|rule| RegisterError::Invalid(format!("{field}: {rule}: {u}")))?;
    }
    if out.join(",").len() > MAX_URIS_CHARS {
        return Err(RegisterError::Invalid(format!(
            "{field} must not be longer than {MAX_URIS_CHARS} characters in total"
        )));
    }
    Ok(out)
}

/// Validates and stores a module. The secret is returned once and only its hash is kept.
pub async fn register(db: &PgPool, cfg: &Config, r: Registration, registered_by: &str) -> Result<Registered, RegisterError> {
    let mut fields = Vec::new();
    let client_id = r.client_id.unwrap_or_default().trim().to_string();
    if client_id.is_empty() {
        fields.push(("clientId", "must not be blank".to_string()));
    } else if !valid_client_id(&client_id) {
        fields.push((
            "clientId",
            "must be 2 to 63 lower-case letters, digits or dashes, starting with a letter or digit".to_string(),
        ));
    }
    let name = r.name.unwrap_or_default().trim().to_string();
    if name.is_empty() {
        fields.push(("name", "must not be blank".to_string()));
    } else if name.chars().count() > 200 {
        fields.push(("name", "size must be between 0 and 200".to_string()));
    }
    let redirect = r.redirect_uris.unwrap_or_default();
    if redirect.is_empty() {
        fields.push(("redirectUris", "must not be empty".to_string()));
    } else if redirect.iter().any(|u| u.trim().is_empty()) {
        fields.push(("redirectUris", "must not contain blank URLs".to_string()));
    }
    if !fields.is_empty() {
        return Err(RegisterError::Fields(fields));
    }
    // `unicloud-*` names the portal's own audiences; an all-digit id could be mistaken for a user `sub`
    if client_id.eq_ignore_ascii_case(RESERVED_CLIENT_ID)
        || client_id.starts_with("unicloud-")
        || client_id.bytes().all(|b| b.is_ascii_digit())
    {
        return Err(RegisterError::Invalid(format!("clientId: '{client_id}' is reserved")));
    }
    let redirect_uris = clean_uris("redirectUris", redirect, cfg)?;
    let logout = r
        .post_logout_redirect_uris
        .unwrap_or_default()
        .into_iter()
        .filter(|u| !u.trim().is_empty())
        .collect();
    let post_logout_redirect_uris = clean_uris("postLogoutRedirectUris", logout, cfg)?;
    let secret = random_token(32);
    let res = sqlx::query(
        "INSERT INTO oauth_clients (client_id, name, secret_hash, redirect_uris, post_logout_redirect_uris, directory_access, registered_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(&client_id)
    .bind(&name)
    .bind(sha256_hex(&secret))
    .bind(&redirect_uris)
    .bind(&post_logout_redirect_uris)
    .bind(r.directory)
    .bind(registered_by)
    .execute(db)
    .await;
    match res {
        Ok(_) => {
            tracing::info!(client_id, registered_by, directory = r.directory, "SSO module registered");
            Ok(Registered {
                client_id,
                client_secret: secret,
                redirect_uris,
                post_logout_redirect_uris,
                directory: r.directory,
            })
        }
        Err(e) if is_unique_violation(&e) => Err(RegisterError::Duplicate(format!(
            "SSO client already exists with clientId: '{client_id}'"
        ))),
        Err(e) => Err(RegisterError::Db(e)),
    }
}

// ---------- Admin page ----------

#[derive(Deserialize)]
pub struct ModuleForm {
    client_id: String,
    name: String,
    redirect_uris: String,
    #[serde(default)]
    post_logout_redirect_uris: String,
    #[serde(default)]
    directory: Option<String>,
}

fn lines(s: &str) -> Vec<String> {
    s.lines().map(str::trim).filter(|l| !l.is_empty()).map(String::from).collect()
}

pub async fn modules_page(State(st): State<AppState>, cu: CurrentUser) -> AppResult<Markup> {
    require_admin(&cu)?;
    modules_view(&st, &cu, None, None).await
}

/// The answer shows a secret once: never cache it
pub async fn modules_create(State(st): State<AppState>, cu: CurrentUser, Form(f): Form<ModuleForm>) -> AppResult<axum::response::Response> {
    use axum::response::IntoResponse;
    let page = modules_create_page(&st, &cu, f).await?;
    Ok(([(axum::http::header::CACHE_CONTROL, "no-store")], page).into_response())
}

async fn modules_create_page(st: &AppState, cu: &CurrentUser, f: ModuleForm) -> AppResult<Markup> {
    require_admin(cu)?;
    let reg = Registration {
        client_id: Some(f.client_id),
        name: Some(f.name),
        redirect_uris: Some(lines(&f.redirect_uris)),
        post_logout_redirect_uris: Some(lines(&f.post_logout_redirect_uris)),
        directory: f.directory.is_some(),
    };
    match register(&st.db, &st.cfg, reg, &format!("user:{}", cu.user.id)).await {
        Ok(r) => modules_view(st, cu, None, Some(&r)).await,
        Err(RegisterError::Fields(f)) => {
            let msg: Vec<String> = f.into_iter().map(|(k, m)| format!("{k}: {m}")).collect();
            modules_view(st, cu, Some(&msg.join("; ")), None).await
        }
        Err(RegisterError::Invalid(m) | RegisterError::Duplicate(m)) => modules_view(st, cu, Some(&m), None).await,
        Err(RegisterError::Db(e)) => Err(e.into()),
    }
}

fn require_admin(cu: &CurrentUser) -> AppResult<()> {
    if cu.user.role == Role::Admin {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

async fn modules_view(st: &AppState, cu: &CurrentUser, error: Option<&str>, created: Option<&Registered>) -> AppResult<Markup> {
    let rows: Vec<(String, String, Vec<String>, bool, String)> = sqlx::query_as(
        "SELECT client_id, name, redirect_uris, directory_access, to_char(created_at, 'YYYY-MM-DD') FROM oauth_clients ORDER BY created_at DESC",
    )
    .fetch_all(&st.db)
    .await?;
    Ok(layout(
        "Модули",
        Some(&cu.nav()),
        html! {
            (page_title("Модули за единен вход", Some("Регистрирани продукти на другите екипи. Тайната се показва само веднъж.")))
            @if let Some(r) = created {
                section.card.secret {
                    h2 { "Модулът " code { (r.client_id) } " е регистриран" }
                    p { "Копирайте тайната сега и я изпратете на екипа по защитен канал. Порталът пази само нейния хеш." }
                    pre.secret-value { (r.client_secret) }
                }
            }
            div.grid-2 {
                section.card {
                    h2 { "Нов модул" }
                    @if let Some(e) = error { (flash("error", e)) }
                    form method="post" action="/admin/modules" hx-boost="false" {
                        (csrf_field(&cu.csrf))
                        (field("Client id (напр. library)", "client_id", "text", "", true))
                        (field("Име", "name", "text", "", true))
                        label.field { span { "Адреси за връщане (по един на ред)" } textarea name="redirect_uris" rows="3" required {} }
                        label.field { span { "Адреси след изход (по желание)" } textarea name="post_logout_redirect_uris" rows="2" {} }
                        label.check { input type="checkbox" name="directory" value="on"; " Достъп до справочника (client_credentials, scope directory)" }
                        button.primary type="submit" { "Регистрирай" }
                    }
                }
                section.card {
                    h2 { "Регистрирани модули" }
                    @if rows.is_empty() { (empty("Няма регистрирани модули.")) } @else {
                        table.table {
                            thead { tr { th { "Client id" } th { "Адреси" } th { "Справочник" } th { "От" } } }
                            tbody {
                                @for (id, name, uris, dir, at) in &rows {
                                    tr {
                                        td { code { (id) } br; span.muted.small { (name) } }
                                        td.small { @for u in uris { (u) br; } }
                                        td { @if *dir { "да" } @else { "не" } }
                                        td { (at) }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::valid_client_id;

    #[test]
    fn client_id_rules() {
        assert!(valid_client_id("library"));
        assert!(valid_client_id("m2"));
        assert!(!valid_client_id("-bad"));
        assert!(!valid_client_id("a"));
        assert!(!valid_client_id("Library"));
        assert!(!valid_client_id(&"a".repeat(64)));
    }
}
