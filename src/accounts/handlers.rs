//! Sign-in, sign-out and the first-password change.

use axum::Form;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use maud::{Markup, html};
use serde::Deserialize;
use tower_sessions::Session;

use super::{password, throttle};
use crate::error::{AppError, AppResult};
use crate::spa::APP_HOME;
use crate::state::AppState;
use crate::ui::components::{csrf_field, field, flash, layout};
use crate::web::session::{CurrentUser, KEY_PENDING_AUTHORIZE, csrf_token, session_user, sign_in};

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    pub sso: Option<String>,
    /// Where in the app to return after sign-in (`/app/...` only)
    pub next: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
    #[serde(default)]
    pub next: Option<String>,
}

const BAD_LOGIN: &str = "Грешен имейл или парола.";

pub async fn login_page(State(st): State<AppState>, session: Session, Query(q): Query<LoginQuery>) -> AppResult<Response> {
    if session_user(&st, &session).await?.is_some() {
        return Ok(Redirect::to(&after_login_target(&session, q.next.as_deref()).await?).into_response());
    }
    let csrf = csrf_token(&session).await?;
    let next = app_path(q.next.as_deref());
    Ok(login_view(&csrf, None, "", q.sso.is_some(), next).into_response())
}

/// An in-app return path, or nothing. Never another site: no scheme, no `//`, no backslash.
fn app_path(next: Option<&str>) -> Option<&str> {
    next.filter(|n| n.starts_with(APP_HOME) && !n.contains("//") && !n.contains('\\') && n.len() <= 512)
}

fn login_view(csrf: &str, error: Option<&str>, email: &str, sso: bool, next: Option<&str>) -> Markup {
    layout(
        "Вход",
        None,
        html! {
            section.card.narrow.login {
                p.kicker { "Единен университетски портал" }
                h1 { "Вход в UniCloud" }
                @if sso { p.muted { "Модул на UniCloud иска да ви идентифицира. Влезте веднъж тук и ще бъдете върнати обратно." } }
                @if let Some(e) = error { (flash("error", e)) }
                form method="post" action="/login" hx-boost="false" {
                    (csrf_field(csrf))
                    @if let Some(n) = next { input type="hidden" name="next" value=(n); }
                    (field("Имейл", "email", "email", email, true))
                    (field("Парола", "password", "password", "", true))
                    button.primary type="submit" { "Вход" }
                }
                p.muted.small { "Нямате профил? Студентите получават профил от учебен отдел, служителите от администратор." }
            }
        },
    )
}

pub async fn login_submit(State(st): State<AppState>, session: Session, Form(f): Form<LoginForm>) -> AppResult<Response> {
    let email = f.email.trim().to_lowercase();
    if throttle::is_locked(&email) {
        let csrf = csrf_token(&session).await?;
        let msg = "Твърде много неуспешни опити. Опитайте отново след 15 минути.";
        return Ok((
            axum::http::StatusCode::TOO_MANY_REQUESTS,
            login_view(&csrf, Some(msg), &email, false, app_path(f.next.as_deref())),
        )
            .into_response());
    }
    let row: Option<(i64, String, bool)> = sqlx::query_as("SELECT id, password_hash, enabled FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&st.db)
        .await?;
    let ok = match row {
        Some((id, hash, enabled)) => (password::verify(f.password, hash).await && enabled).then_some(id),
        None => {
            // Same work as a real check, so the answer time does not reveal which emails exist
            password::verify_dummy(f.password).await;
            None
        }
    };
    let Some(user_id) = ok else {
        throttle::record_failure(&email);
        let csrf = csrf_token(&session).await?;
        return Ok((
            axum::http::StatusCode::UNAUTHORIZED,
            login_view(&csrf, Some(BAD_LOGIN), &email, false, app_path(f.next.as_deref())),
        )
            .into_response());
    };
    throttle::clear(&email);
    sign_in(&session, user_id).await?;
    tracing::info!(user_id, "signed in");
    Ok(Redirect::to(&after_login_target(&session, f.next.as_deref()).await?).into_response())
}

/// The waiting SSO login, if one is stored; else the asked-for app page; else the app's start
async fn after_login_target(session: &Session, next: Option<&str>) -> AppResult<String> {
    let pending: Option<String> = session.remove(KEY_PENDING_AUTHORIZE).await.map_err(anyhow::Error::from)?;
    Ok(pending
        .filter(|p| p.starts_with("/oauth2/authorize?"))
        .or_else(|| app_path(next).map(String::from))
        .unwrap_or_else(|| APP_HOME.into()))
}

pub async fn logout(State(st): State<AppState>, session: Session) -> AppResult<Response> {
    crate::web::session::end_session(&st, &session).await?;
    Ok(Redirect::to("/login").into_response())
}

#[derive(Deserialize)]
pub struct PasswordForm {
    pub password: String,
    pub new_password: String,
    pub new_password2: String,
}

pub async fn password_page(cu: CurrentUser) -> Markup {
    password_view(&cu, None)
}

fn password_view(cu: &CurrentUser, error: Option<&str>) -> Markup {
    let forced = cu.user.must_change_password;
    layout(
        "Смяна на парола",
        Some(&cu.nav()),
        html! {
            section.card.narrow {
                h1 { "Смяна на парола" }
                @if forced { p { "Това е първото ви влизане. Изберете своя парола, за да продължите." } }
                @if let Some(e) = error { (flash("error", e)) }
                form method="post" action="/password" hx-boost="false" {
                    (csrf_field(&cu.csrf))
                    (field("Текуща парола", "password", "password", "", true))
                    (field("Нова парола (поне 8 знака)", "new_password", "password", "", true))
                    (field("Повторете новата парола", "new_password2", "password", "", true))
                    button.primary type="submit" { "Запази" }
                }
            }
        },
    )
}

pub async fn password_submit(
    State(st): State<AppState>,
    headers: HeaderMap,
    cu: CurrentUser,
    Form(f): Form<PasswordForm>,
) -> AppResult<Response> {
    let (hash,): (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
        .bind(cu.user.id)
        .fetch_one(&st.db)
        .await?;
    if !password::verify(f.password.clone(), hash).await {
        return Ok(password_view(&cu, Some("Текущата парола е грешна.")).into_response());
    }
    if f.new_password != f.new_password2 {
        return Ok(password_view(&cu, Some("Новите пароли не съвпадат.")).into_response());
    }
    if f.new_password == f.password {
        return Ok(password_view(&cu, Some("Новата парола трябва да е различна от текущата.")).into_response());
    }
    if let Err(msg) = password::check_strength(&f.new_password) {
        return Ok(password_view(&cu, Some(msg)).into_response());
    }
    let new_hash = password::hash(f.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = $1, must_change_password = FALSE WHERE id = $2")
        .bind(new_hash)
        .bind(cu.user.id)
        .execute(&st.db)
        .await?;
    // Tokens issued before the change stop working at /userinfo
    sqlx::query("UPDATE oauth_grants SET revoked = TRUE WHERE user_id = $1")
        .bind(cu.user.id)
        .execute(&st.db)
        .await?;
    let target = after_login_target(&cu.session, None).await?;
    Ok(crate::web::redirect(&headers, &target))
}

/// Used by forms elsewhere: the error for a missing permission
pub fn forbidden() -> AppError {
    AppError::Forbidden
}
