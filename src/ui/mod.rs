//! The Bulgarian UI: Maud components and pages. Handlers answer htmx requests with fragments.

pub mod catalog;
pub mod components;
pub mod inspect;
pub mod pages;
pub mod staging_page;
pub mod teach;

use crate::access::Permission;
use crate::accounts::model::User;

/// The navigation a user sees. The server checks permissions again on every request.
pub fn nav_links(u: &User) -> Vec<(&'static str, &'static str)> {
    let mut links = vec![("/home", "Начало")];
    if u.can(Permission::StudentProfileReadOwn) {
        links.push(("/profile", "Профил"));
    }
    if u.can(Permission::CourseOfferingRead) {
        links.push(("/courses", "Курсове"));
    }
    if u.can(Permission::EnrollmentRequestCreate) {
        links.push(("/enrollments", "Моите записвания"));
    }
    if u.can(Permission::EnrollmentRequestAcceptReject) {
        links.push(("/teach", "Моите курсове"));
    }
    if u.can(Permission::StudentProfileReadFaculty) {
        links.push(("/faculty", "Факултет"));
    }
    if u.can(Permission::StudentCreate) || u.can(Permission::StudentProfileReadAny) {
        links.push(("/inspect/students", "Студенти"));
    }
    if u.can(Permission::CourseCreateModify) || u.can(Permission::CourseOfferingCreateModify) {
        links.push(("/inspect/courses", "Дисциплини"));
    }
    if u.can(Permission::StaffCreate) {
        links.push(("/admin/users", "Служители"));
    }
    if u.role == crate::access::Role::Admin {
        links.push(("/admin/modules", "Модули"));
    }
    links
}
