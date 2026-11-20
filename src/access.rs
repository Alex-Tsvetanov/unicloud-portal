//! Roles and permissions. Bit positions are part of the published contract: never reorder.

use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Role {
    Admin,
    Inspector,
    Teacher,
    Dean,
    Student,
}

impl Role {
    /// Display and listing order (staging tools)
    pub const ALL: [Role; 5] = [Role::Admin, Role::Inspector, Role::Teacher, Role::Dean, Role::Student];

    pub fn as_str(self) -> &'static str {
        match self {
            Role::Admin => "ADMIN",
            Role::Inspector => "INSPECTOR",
            Role::Teacher => "TEACHER",
            Role::Dean => "DEAN",
            Role::Student => "STUDENT",
        }
    }

    pub fn parse(s: &str) -> Option<Role> {
        Role::ALL.into_iter().find(|r| r.as_str().eq_ignore_ascii_case(s.trim()))
    }

    pub fn label_bg(self) -> &'static str {
        match self {
            Role::Admin => "Администратор",
            Role::Inspector => "Инспектор",
            Role::Teacher => "Преподавател",
            Role::Dean => "Декан",
            Role::Student => "Студент",
        }
    }

    pub fn default_permissions(self) -> &'static [Permission] {
        use Permission::*;
        match self {
            Role::Admin => &[StaffCreate],
            Role::Inspector => &[
                StudentCreate,
                StudentProfileReadAny,
                CourseCreateModify,
                CourseOfferingRead,
                CourseOfferingCreateModify,
            ],
            Role::Teacher => &[CourseOfferingRead, StudentProfileReadEnrolled, EnrollmentRequestAcceptReject],
            Role::Dean => &[StudentProfileReadFaculty, CourseOfferingRead],
            Role::Student => &[StudentProfileReadOwn, CourseOfferingRead, EnrollmentRequestCreate],
        }
    }

    pub fn default_mask(self) -> i64 {
        mask_of(self.default_permissions())
    }

    /// Teachers, deans and inspectors belong to one faculty
    pub fn needs_faculty(self) -> bool {
        matches!(self, Role::Teacher | Role::Dean | Role::Inspector)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Permission {
    StaffCreate = 0,
    StudentCreate = 1,
    StudentProfileReadOwn = 2,
    StudentProfileReadAny = 3,
    StudentProfileReadEnrolled = 4,
    CourseCreateModify = 5,
    CourseOfferingRead = 6,
    CourseOfferingCreateModify = 7,
    EnrollmentRequestCreate = 8,
    EnrollmentRequestAcceptReject = 9,
    StudentProfileReadFaculty = 10,
}

impl Permission {
    /// Bit order
    pub const ALL: [Permission; 11] = [
        Permission::StaffCreate,
        Permission::StudentCreate,
        Permission::StudentProfileReadOwn,
        Permission::StudentProfileReadAny,
        Permission::StudentProfileReadEnrolled,
        Permission::CourseCreateModify,
        Permission::CourseOfferingRead,
        Permission::CourseOfferingCreateModify,
        Permission::EnrollmentRequestCreate,
        Permission::EnrollmentRequestAcceptReject,
        Permission::StudentProfileReadFaculty,
    ];

    pub fn bit(self) -> i64 {
        1i64 << (self as u32)
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Permission::StaffCreate => "STAFF_CREATE",
            Permission::StudentCreate => "STUDENT_CREATE",
            Permission::StudentProfileReadOwn => "STUDENT_PROFILE_READ_OWN",
            Permission::StudentProfileReadAny => "STUDENT_PROFILE_READ_ANY",
            Permission::StudentProfileReadEnrolled => "STUDENT_PROFILE_READ_ENROLLED",
            Permission::CourseCreateModify => "COURSE_CREATE_MODIFY",
            Permission::CourseOfferingRead => "COURSE_OFFERING_READ",
            Permission::CourseOfferingCreateModify => "COURSE_OFFERING_CREATE_MODIFY",
            Permission::EnrollmentRequestCreate => "ENROLLMENT_REQUEST_CREATE",
            Permission::EnrollmentRequestAcceptReject => "ENROLLMENT_REQUEST_ACCEPT_REJECT",
            Permission::StudentProfileReadFaculty => "STUDENT_PROFILE_READ_FACULTY",
        }
    }

    pub fn parse(s: &str) -> Option<Permission> {
        Permission::ALL.into_iter().find(|p| p.as_str() == s.trim())
    }
}

pub fn mask_of(perms: &[Permission]) -> i64 {
    perms.iter().fold(0, |m, p| m | p.bit())
}

/// Names in bit order, as they go on the wire
pub fn names_of(mask: i64) -> Vec<&'static str> {
    Permission::ALL
        .into_iter()
        .filter(|p| mask & p.bit() != 0)
        .map(Permission::as_str)
        .collect()
}

pub fn has(mask: i64, p: Permission) -> bool {
    mask & p.bit() != 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn role_masks_match_the_java_portal() {
        assert_eq!(Role::Admin.default_mask(), 1);
        assert_eq!(Role::Inspector.default_mask(), 234);
        assert_eq!(Role::Teacher.default_mask(), 592);
        assert_eq!(Role::Student.default_mask(), 324);
        assert_eq!(Role::Dean.default_mask(), (1 << 10) | (1 << 6));
    }

    #[test]
    fn names_are_in_bit_order() {
        assert_eq!(
            names_of(Role::Student.default_mask()),
            ["STUDENT_PROFILE_READ_OWN", "COURSE_OFFERING_READ", "ENROLLMENT_REQUEST_CREATE"]
        );
    }
}
