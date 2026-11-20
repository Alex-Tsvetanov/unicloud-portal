//! Sign-in, sign-out and the first-password change.

use axum::Form;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use maud::{Markup, html};
use serde::Deserialize;
use tower_sessions::Session;

use super::password;
use crate::error::{AppError, AppResult};
use crate::state::AppState;
use crate::ui::components::{csrf_field, field, flash, layout};
use crate::web::session::{CurrentUser, KEY_PENDING_AUTHORIZE, csrf_token, session_user, sign_in};

#[derive(Deserialize, Default)]
pub struct LoginQuery {
    pub sso: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginForm {
    pub email: String,
    pub password: String,
}

const BAD_LOGIN: &str = "Грешен имейл или парола.";

pub async fn login_page(State(st): State<AppState>, session: Session, Query(q): Query<LoginQuery>) -> AppResult<Response> {
    if session_user(&st, &session).await?.is_some() {
        return Ok(Redirect::to(&after_login_target(&session).await?).into_response());
    }
    let csrf = csrf_token(&session).await?;
    Ok(login_view(&csrf, None, "", q.sso.is_some()).into_response())
}

fn login_view(csrf: &str, error: Option<&str>, email: &str, sso: bool) -> Markup {
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
    let row: Option<(i64, String, bool)> = sqlx::query_as("SELECT id, password_hash, enabled FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&st.db)
        .await?;
    let ok = match row {
        Some((id, hash, enabled)) => (password::verify(f.password, hash).await && enabled).then_some(id),
        None => None,
    };
    let Some(user_id) = ok else {
        let csrf = csrf_token(&session).await?;
        return Ok((
            axum::http::StatusCode::UNAUTHORIZED,
            login_view(&csrf, Some(BAD_LOGIN), &email, false),
        )
            .into_response());
    };
    sign_in(&session, user_id).await?;
    tracing::info!(user_id, "signed in");
    Ok(Redirect::to(&after_login_target(&session).await?).into_response())
}

/// The waiting SSO login, if one is stored, or the home page
async fn after_login_target(session: &Session) -> AppResult<String> {
    let pending: Option<String> = session.remove(KEY_PENDING_AUTHORIZE).await.map_err(anyhow::Error::from)?;
    Ok(pending
        .filter(|p| p.starts_with("/oauth2/authorize?"))
        .unwrap_or_else(|| "/".into()))
}

pub async fn logout(session: Session) -> AppResult<Response> {
    session.flush().await.map_err(anyhow::Error::from)?;
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
    if !password::verify(f.password, hash).await {
        return Ok(password_view(&cu, Some("Текущата парола е грешна.")).into_response());
    }
    if f.new_password != f.new_password2 {
        return Ok(password_view(&cu, Some("Новите пароли не съвпадат.")).into_response());
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
    let target = after_login_target(&cu.session).await?;
    Ok(crate::web::redirect(&headers, &target))
}

/// Used by forms elsewhere: the error for a missing permission
pub fn forbidden() -> AppError {
    AppError::Forbidden
}
