//! The `/staging` page. Forms post JSON to the staging API (assets/app.js); the users table is an
//! htmx fragment that reloads after each change.

use maud::{Markup, html};

use crate::access::{Permission, Role};
use crate::ui::components::layout;

pub fn users_table(rows: &[(i64, String, String, String, Option<String>, Vec<&'static str>)]) -> Markup {
    html! {
        table.table #users-table {
            thead { tr { th { "id" } th { "Потребител" } th { "Роля" } th { "Факултет" } th { "Права" } } }
            tbody {
                @for (id, email, name, role, fac, perms) in rows {
                    tr {
                        td { (id) }
                        td { (name) br; span.muted.small { (email) } }
                        td { code { (role) } }
                        td { (fac.as_deref().unwrap_or("–")) }
                        td.small {
                            form.perm-form data-user=(id) {
                                @for p in Permission::ALL {
                                    label.chip { input type="checkbox" name="permissions" value=(p.as_str()) checked[perms.contains(&p.as_str())]; (p.as_str()) }
                                }
                                button.link type="submit" { "Запази правата" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn render(table: Markup, specialties: &[(i32, String, String)], faculties: &[(String, String)]) -> Markup {
    layout(
        "Staging",
        None,
        html! {
            div.page-head {
                p.kicker { "Само за staging" }
                h1 { "Инструменти за екипите на модулите" }
                p.warning { "Внимание: всички данни тук са общи за всички екипи и се изтриват, когато безплатната база данни бъде заменена (на всеки 30 дни). Използвайте измислени хора и пароли." }
            }
            div.grid-2 {
                section.card {
                    h2 { "Регистрирайте модул за единен вход" }
                    form.json-form data-endpoint="/api/v1/staging/sso-clients" data-kind="client" {
                        label.field { span { "Client id" } input name="clientId" required placeholder="module-x"; }
                        label.field { span { "Име" } input name="name" required; }
                        label.field { span { "Адреси за връщане (по един на ред; на staging работи всеки)" } textarea name="redirectUris" rows="2" { "http://localhost:3000/auth/callback" } }
                        label.field { span { "Адреси след изход (по желание)" } textarea name="postLogoutRedirectUris" rows="2" {} }
                        label.check { input type="checkbox" name="directory"; " Достъп до справочника (scope directory)" }
                        button.primary type="submit" { "Регистрирай" }
                    }
                    div.result {}
                }
                section.card {
                    h2 { "Създайте тестов потребител" }
                    form.json-form data-endpoint="/api/v1/staging/users" data-kind="user" {
                        label.field { span { "Имейл" } input name="email" type="email" required; }
                        label.field { span { "Парола (8 до 72 знака)" } input name="password" required minlength="8" maxlength="72"; }
                        label.field { span { "Име" } input name="firstName" required; }
                        label.field { span { "Фамилия" } input name="lastName" required; }
                        label.field { span { "Роля" } select name="role" { @for r in Role::ALL { option value=(r.as_str()) selected[r == Role::Student] { (r.as_str()) } } } }
                        label.field { span { "Факултет (за преподавател, декан, инспектор)" } select name="faculty" { option value="" { "–" } @for (c, n) in faculties { option value=(c) { (c) " · " (n) } } } }
                        fieldset.student {
                            legend { "Академичен профил (само за студент)" }
                            label.field { span { "Факултетен номер" } input name="facultyNumber"; }
                            label.field { span { "Специалност" } select name="specialty" { @for (id, f, n) in specialties { option value=(id) { (f) " · " (n) } } } }
                            label.field { span { "Курс" } input name="studyYear" type="number" min="1" max="6" value="1"; }
                            label.field { span { "Статус" } select name="status" { option { "ACTIVE" } option { "INTERRUPTED" } option { "GRADUATED" } } }
                        }
                        p.muted.small { "Правата по подразбиране за ролята се задават автоматично. Променете ги после в таблицата." }
                        button.primary type="submit" { "Създай" }
                    }
                    div.result {}
                }
            }
            section.card {
                div.row-between {
                    h2 { "Потребители" }
                    button.link hx-get="/staging/users-table" hx-target="#users-table" hx-swap="outerHTML" { "Обнови" }
                }
                (table)
            }
        },
    )
}
