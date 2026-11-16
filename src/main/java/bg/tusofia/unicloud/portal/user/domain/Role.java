package bg.tusofia.unicloud.portal.user.domain;

public enum Role {

    ADMIN(
            Permission.STAFF_CREATE
    ),

    INSPECTOR(
            Permission.STUDENT_CREATE,
            Permission.STUDENT_PROFILE_READ_ANY,
            Permission.COURSE_CREATE_MODIFY,
            Permission.COURSE_OFFERING_READ,
            Permission.COURSE_OFFERING_CREATE_MODIFY
    ),

    TEACHER(
            Permission.COURSE_OFFERING_READ,
            Permission.STUDENT_PROFILE_READ_ENROLLED,
            Permission.ENROLLMENT_REQUEST_ACCEPT_REJECT
    ),

    STUDENT(
            Permission.STUDENT_PROFILE_READ_OWN,
            Permission.COURSE_OFFERING_READ,
            Permission.ENROLLMENT_REQUEST_CREATE
    );

    private final long defaultPermissions;

    Role(Permission... permissions) {
        long mask = 0;

        for (Permission permission : permissions) {
            mask |= permission.getMask();
        }

        this.defaultPermissions = mask;
    }

    public long getDefaultPermissions() {
        return defaultPermissions;
    }
}
