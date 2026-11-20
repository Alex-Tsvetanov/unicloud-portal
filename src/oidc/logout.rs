//! `GET /connect/logout`: RP-initiated logout. Ends the browser's SSO session, then returns to the
//! module's registered logout URL (docs/sso.md, "Logout").

use axum::extract::{Query, State};
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use jsonwebtoken::{Algorithm, Validation};
use serde::Deserialize;
use tower_sessions::Session;
use uuid::Uuid;

use super::{clients, found, issuer, policy, with_query};
use crate::error::AppResult;
use crate::state::AppState;
use crate::ui::components::error_page;

#[derive(Deserialize)]
pub struct LogoutQuery {
    id_token_hint: Option<String>,
    client_id: Option<String>,
    post_logout_redirect_uri: Option<String>,
    state: Option<String>,
}

#[derive(Deserialize)]
struct HintClaims {
    jti: String,
    azp: Option<String>,
}

fn bad(msg: &str) -> Response {
    (StatusCode::BAD_REQUEST, error_page(StatusCode::BAD_REQUEST, msg)).into_response()
}

pub async fn logout(State(st): State<AppState>, headers: HeaderMap, session: Session, Query(q): Query<LogoutQuery>) -> AppResult<Response> {
    // Which module asks? From the ID token hint (kept resolvable for 24 hours), or from client_id.
    let mut client_id = q.client_id.clone();
    let mut verified_hint = false;
    if let Some(hint) = q.id_token_hint.as_deref().filter(|h| !h.is_empty()) {
        let mut v = Validation::new(Algorithm::RS256);
        v.set_issuer(&[issuer(&st, &headers)]);
        v.validate_aud = false;
        v.validate_exp = false;
        v.set_required_spec_claims(&["iss"]);
        let Ok(data) = jsonwebtoken::decode::<HintClaims>(hint, st.keys.decoding(), &v) else {
            return Ok(bad("Невалиден id_token_hint."));
        };
        let jti = Uuid::parse_str(&data.claims.jti).ok();
        let grant: Option<(String,)> = match jti {
            Some(j) => {
                sqlx::query_as("SELECT client_id FROM oauth_grants WHERE id_token_jti = $1")
                    .bind(j)
                    .fetch_optional(&st.db)
                    .await?
            }
            None => None,
        };
        let Some((grant_client,)) = grant.or(data.claims.azp.map(|a| (a,))) else {
            return Ok(bad("Входът е твърде стар или непознат."));
        };
        if client_id.as_deref().is_some_and(|c| c != grant_client) {
            return Ok(bad("client_id не съвпада с id_token_hint."));
        }
        client_id = Some(grant_client);
        verified_hint = true;
    }

    let target = match q.post_logout_redirect_uri.as_deref() {
        Some(uri) => {
            let Some(client) = (match client_id.as_deref() {
                Some(id) => clients::find(&st.db, id).await?,
                None => None,
            }) else {
                return Ok(bad("Непознат модул."));
            };
            if !policy::trusted(
                st.cfg.callback_policy,
                &st.cfg.allowed_callback_hosts,
                &client.post_logout_redirect_uris,
                uri,
            ) {
                return Ok(bad("Адресът след изход не е разрешен за този модул."));
            }
            let state: Vec<(&str, &str)> = q.state.as_deref().map(|s| vec![("state", s)]).unwrap_or_default();
            with_query(uri, &state)
        }
        None => "/login".to_string(),
    };
    // Without a valid ID token hint any website could log the user out with a link: ask first
    if !verified_hint {
        let csrf = crate::web::session::csrf_token(&session).await?;
        return Ok(crate::ui::components::confirm_logout(&csrf).into_response());
    }
    crate::web::session::end_session(&st, &session).await?;
    Ok(found(&target))
}
