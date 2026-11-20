//! Maud components shared by every page. A component is a function that returns `Markup`.

use axum::http::StatusCode;
use maud::{DOCTYPE, Markup, PreEscaped, html};

/// What the layout needs to know about the signed-in user
pub struct NavUser<'a> {
    pub name: String,
    pub role_label: &'a str,
    pub csrf: &'a str,
}

pub fn layout(title: &str, user: Option<&NavUser>, body: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html lang="bg" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                @if let Some(u) = user { meta name="csrf-token" content=(u.csrf); }
                title { (title) " · UniCloud" }
                link rel="stylesheet" href="/assets/app.css";
                script src="/assets/htmx.min.js" defer {}
                script src="/assets/app.js" defer {}
            }
            body hx-boost="true" {
                header.topbar {
                    a.brand href="/app/" { span.brand-mark { "U" } "UniCloud" }
                    @if let Some(u) = user {
                        nav aria-label="Основна навигация" { a href="/app/" { "Към портала" } }
                        div.who {
                            span.who-name { (u.name) }
                            span.badge { (u.role_label) }
                            form method="post" action="/logout" hx-boost="false" {
                                input type="hidden" name="csrf" value=(u.csrf);
                                button.link type="submit" { "Изход" }
                            }
                        }
                    }
                }
                main #main { (body) }
                footer.footer { "UniCloud · Технически университет – София" }
            }
        }
    }
}

pub fn page_title(title: &str, subtitle: Option<&str>) -> Markup {
    html! {
        div.page-head {
            h1 { (title) }
            @if let Some(s) = subtitle { p.muted { (s) } }
        }
    }
}

pub fn flash(kind: &str, msg: &str) -> Markup {
    html! { div class={"flash flash-" (kind)} role="status" { (msg) } }
}

pub fn field(label: &str, name: &str, kind: &str, value: &str, required: bool) -> Markup {
    html! {
        label.field {
            span { (label) }
            input type=(kind) name=(name) value=(value) required[required] autocomplete=(autocomplete(name));
        }
    }
}

fn autocomplete(name: &str) -> &'static str {
    match name {
        "email" => "username",
        "password" => "current-password",
        "new_password" | "new_password2" => "new-password",
        _ => "off",
    }
}

pub fn select(label: &str, name: &str, options: &[(String, String)], selected: &str) -> Markup {
    html! {
        label.field {
            span { (label) }
            select name=(name) {
                @for (value, text) in options {
                    option value=(value) selected[value == selected] { (text) }
                }
            }
        }
    }
}

pub fn csrf_field(token: &str) -> Markup {
    html! { input type="hidden" name="csrf" value=(token); }
}

pub fn status_badge(status: &str) -> Markup {
    let (cls, text) = match status {
        "ACTIVE" => ("ok", "Действащ"),
        "INTERRUPTED" => ("warn", "Прекъснал"),
        "GRADUATED" => ("info", "Дипломиран"),
        "ENROLLED" => ("ok", "Записан"),
        "CANCELLED" => ("muted", "Отказан"),
        "REJECTED" => ("warn", "Отхвърлен"),
        other => ("muted", other),
    };
    html! { span class={"badge badge-" (cls)} { (text) } }
}

/// Places left as a small meter
pub fn seats(enrolled: i32, capacity: i32) -> Markup {
    let free = (capacity - enrolled).max(0);
    let pct = if capacity > 0 { enrolled * 100 / capacity } else { 100 };
    html! {
        div.seats title={(free) " свободни от " (capacity)} {
            div.seats-bar { div.seats-fill style={"width:" (pct) "%"} {} }
            span { (free) " / " (capacity) }
        }
    }
}

pub fn error_page(status: StatusCode, msg: &str) -> Markup {
    layout(
        "Грешка",
        None,
        html! {
            section.card.narrow {
                p.kicker { "Грешка " (status.as_u16()) }
                h1 { (msg) }
                p { a href="/" { "Към началната страница" } }
            }
        },
    )
}

pub fn empty(msg: &str) -> Markup {
    html! { p.empty { (msg) } }
}

pub fn raw(s: &str) -> PreEscaped<&str> {
    PreEscaped(s)
}

/// Logout asked for without a valid ID token hint: the user confirms with a CSRF-protected form
pub fn confirm_logout(csrf: &str) -> Markup {
    layout(
        "Изход",
        None,
        html! {
            section.card.narrow {
                h1 { "Изход от UniCloud?" }
                p { "Ще излезете от портала и от всички модули в този браузър." }
                form method="post" action="/logout" hx-boost="false" {
                    (csrf_field(csrf))
                    button.primary type="submit" { "Изход" }
                    " "
                    a href="/" { "Отказ" }
                }
            }
        },
    )
}
