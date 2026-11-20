//! `GET /oauth2/authorize`: the authorization code flow with PKCE (docs/sso.md, "The flow").
//! Order of checks: an untrusted client or callback gets a plain 400 and no redirect; once the
//! callback is trusted, every other error goes back to it.

use axum::extract::{Query, RawQuery, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use time::OffsetDateTime;
use tower_sessions::Session;
use uuid::Uuid;

use super::{CODE_TTL_SECS, USER_SCOPES, clients, found, policy, with_query};
use crate::crypto::{random_token, sha256_hex};
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::ui::components::error_page;
use crate::web::session::{KEY_PENDING_AUTHORIZE, KEY_SID, session_user};

#[derive(Deserialize, Debug)]
pub struct AuthorizeQuery {
    response_type: Option<String>,
    client_id: Option<String>,
    redirect_uri: Option<String>,
    scope: Option<String>,
    state: Option<String>,
    nonce: Option<String>,
    code_challenge: Option<String>,
    code_challenge_method: Option<String>,
}

fn plain_400(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, error_page(StatusCode::BAD_REQUEST, msg)).into_response()
}

fn back_with_error(cb: &str, code: &str, desc: &str, state: Option<&str>) -> Response {
    let mut p = vec![("error", code), ("error_description", desc)];
    if let Some(s) = state {
        p.push(("state", s));
    }
    found(&with_query(cb, &p))
}

pub async fn authorize(
    State(st): State<AppState>,
    session: Session,
    RawQuery(raw): RawQuery,
    Query(q): Query<AuthorizeQuery>,
) -> AppResult<Response> {
    // 1. The module and the callback must be trusted before anything is sent back to it
    let Some(client) = (match q.client_id.as_deref() {
        Some(id) => clients::find(&st.db, id).await?,
        None => None,
    }) else {
        return Ok(plain_400("Непознат модул (client_id). Проверете регистрацията му."));
    };
    let cb = match q.redirect_uri.as_deref() {
        Some(u) => u.to_string(),
        None if client.redirect_uris.len() == 1 => client.redirect_uris[0].clone(),
        None => return Ok(plain_400("Липсва адрес за връщане (redirect_uri).")),
    };
    if !policy::trusted(st.cfg.callback_policy, &st.cfg.allowed_callback_hosts, &client.redirect_uris, &cb) {
        return Ok(plain_400("Адресът за връщане (redirect_uri) не е разрешен за този модул."));
    }
    let state = q.state.as_deref();

    // 2. Request errors go back to the trusted callback
    if q.response_type.as_deref() != Some("code") {
        return Ok(back_with_error(
            &cb,
            "unsupported_response_type",
            "OAuth 2.0 Parameter: response_type",
            state,
        ));
    }
    let scopes: Vec<&str> = q.scope.as_deref().unwrap_or("").split_whitespace().collect();
    if let Some(bad) = scopes.iter().find(|s| !USER_SCOPES.contains(s)) {
        tracing::debug!(scope = bad, "scope refused");
        return Ok(back_with_error(&cb, "invalid_scope", "OAuth 2.0 Parameter: scope", state));
    }
    if !scopes.contains(&"openid") {
        return Ok(back_with_error(
            &cb,
            "invalid_request",
            "OAuth 2.0 Parameter: scope (openid is required)",
            state,
        ));
    }
    let Some(challenge) = q.code_challenge.as_deref().filter(|c| !c.is_empty()) else {
        return Ok(back_with_error(
            &cb,
            "invalid_request",
            "OAuth 2.0 Parameter: code_challenge",
            state,
        ));
    };
    if q.code_challenge_method.as_deref() != Some("S256") || challenge.len() < 43 || challenge.len() > 128 {
        return Ok(back_with_error(
            &cb,
            "invalid_request",
            "OAuth 2.0 Parameter: code_challenge_method",
            state,
        ));
    }

    // 3. Who is the user? Without a session (or before the first password change), sign in first.
    let here = format!("/oauth2/authorize?{}", raw.unwrap_or_default());
    let user = session_user(&st, &session).await?;
    let Some((user, login_at)) = user else {
        session.insert(KEY_PENDING_AUTHORIZE, &here).await.map_err(anyhow::Error::from)?;
        return Ok(found("/login?sso=1"));
    };
    if user.must_change_password {
        session.insert(KEY_PENDING_AUTHORIZE, &here).await.map_err(anyhow::Error::from)?;
        return Ok(found("/password"));
    }

    // 4. Issue a single-use code bound to everything the token request must repeat
    let sid: String = session.get(KEY_SID).await.map_err(anyhow::Error::from)?.unwrap_or_default();
    let grant_id = Uuid::new_v4();
    let code = random_token(32);
    let expires = OffsetDateTime::now_utc() + time::Duration::seconds(CODE_TTL_SECS);
    let mut tx = st.db.begin().await?;
    sqlx::query("INSERT INTO oauth_grants (id, client_id, user_id, scope, session_id) VALUES ($1, $2, $3, $4, $5)")
        .bind(grant_id)
        .bind(&client.client_id)
        .bind(user.id)
        .bind(scopes.join(" "))
        .bind(&sid)
        .execute(&mut *tx)
        .await?;
    sqlx::query(
        "INSERT INTO oauth_codes (code_hash, grant_id, redirect_uri, nonce, code_challenge, auth_time, expires_at) VALUES ($1, $2, $3, $4, $5, $6, $7)",
    )
    .bind(sha256_hex(&code))
    .bind(grant_id)
    .bind(&cb)
    .bind(q.nonce.as_deref())
    .bind(challenge)
    .bind(login_at)
    .bind(expires)
    .execute(&mut *tx)
    .await?;
    tx.commit().await.map_err(AppError::from)?;
    let mut p = vec![("code", code.as_str())];
    if let Some(s) = state {
        p.push(("state", s));
    }
    Ok(found(&with_query(&cb, &p)))
}
