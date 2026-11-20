//! The SSO session (`__Host-UNICLOUD_SSO`): who is signed in, when, and the CSRF token.
//! One cookie serves the portal UI and single sign-on (ADR 0003).

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use time::OffsetDateTime;
use tower_sessions::Session;

use crate::accounts::model::{self, User};
use crate::crypto::random_token;
use crate::state::AppState;
use crate::ui::components::NavUser;
use crate::web::redirect;

pub const KEY_UID: &str = "uid";
pub const KEY_LOGIN_AT: &str = "login_at";
pub const KEY_CSRF: &str = "csrf";
pub const KEY_PENDING_AUTHORIZE: &str = "pending_authorize";
/// The `sid` claim: one value per sign-in, stable while the session lives
pub const KEY_SID: &str = "sid";
/// However busy, a session ends this long after the password was typed
pub const MAX_AGE_SECS: i64 = 8 * 3600;

pub fn now() -> i64 {
    OffsetDateTime::now_utc().unix_timestamp()
}

/// The session's CSRF token, created on first use
pub async fn csrf_token(session: &Session) -> anyhow::Result<String> {
    if let Some(t) = session.get::<String>(KEY_CSRF).await? {
        return Ok(t);
    }
    let t = random_token(32);
    session.insert(KEY_CSRF, &t).await?;
    Ok(t)
}

/// Signs the user in: a new session id (no fixation), the user id, the time, a fresh CSRF token
pub async fn sign_in(session: &Session, user_id: i64) -> anyhow::Result<()> {
    let pending: Option<String> = session.get(KEY_PENDING_AUTHORIZE).await?;
    session.cycle_id().await?;
    session.clear().await;
    session.insert(KEY_UID, user_id).await?;
    session.insert(KEY_LOGIN_AT, now()).await?;
    session.insert(KEY_CSRF, random_token(32)).await?;
    session.insert(KEY_SID, random_token(32)).await?;
    if let Some(p) = pending {
        session.insert(KEY_PENDING_AUTHORIZE, p).await?;
    }
    Ok(())
}

/// Ends the browser's session (portal and SSO) and revokes the module logins made in it
pub async fn end_session(st: &AppState, session: &Session) -> anyhow::Result<()> {
    if let Some(sid) = session.get::<String>(KEY_SID).await? {
        sqlx::query("UPDATE oauth_grants SET revoked = TRUE WHERE session_id = $1")
            .bind(sid)
            .execute(&st.db)
            .await?;
    }
    session.flush().await?;
    Ok(())
}

/// The signed-in user and when they typed the password, if the session is still valid
pub async fn session_user(st: &AppState, session: &Session) -> anyhow::Result<Option<(User, i64)>> {
    let Some(uid) = session.get::<i64>(KEY_UID).await? else {
        return Ok(None);
    };
    let login_at = session.get::<i64>(KEY_LOGIN_AT).await?.unwrap_or(0);
    if now() - login_at > MAX_AGE_SECS {
        session.flush().await?;
        return Ok(None);
    }
    match model::find_enabled(&st.db, uid).await? {
        Some(u) => Ok(Some((u, login_at))),
        None => {
            session.flush().await?;
            Ok(None)
        }
    }
}

/// Extractor: the signed-in user. Without one, the browser goes to `/login`.
/// A user who must change the password goes to `/password` first.
pub struct CurrentUser {
    pub user: User,
    pub csrf: String,
    pub session: Session,
}

impl CurrentUser {
    pub fn nav(&self) -> NavUser<'_> {
        NavUser {
            name: self.user.full_name(),
            role_label: self.user.role.label_bg(),
            csrf: &self.csrf,
        }
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let st = AppState::from_ref(state);
        let session = Session::from_request_parts(parts, state)
            .await
            .map_err(IntoResponse::into_response)?;
        let found = session_user(&st, &session)
            .await
            .map_err(|e| crate::error::AppError::Other(e).into_response())?;
        let Some((user, _)) = found else {
            return Err(redirect(&parts.headers, "/login"));
        };
        let path = parts.uri.path();
        if user.must_change_password && path != "/password" && path != "/logout" {
            return Err(redirect(&parts.headers, "/password"));
        }
        let csrf = csrf_token(&session)
            .await
            .map_err(|e| crate::error::AppError::Other(e).into_response())?;
        Ok(CurrentUser { user, csrf, session })
    }
}
