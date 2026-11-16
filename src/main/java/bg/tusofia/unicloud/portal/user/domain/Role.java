package bg.tusofia.unicloud.portal.user.domain;

import java.util.Collections;
import java.util.EnumSet;
import java.util.Set;

public enum Role {

    ADMIN(EnumSet.of(Permission.STAFF_CREATE)),
    INSPECTOR(EnumSet.of(
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
    )),

    TEACHER(EnumSet.of(
            Permission.COURSE_OFFERING_READ,
            Permission.STUDENT_PROFILE_READ_ENROLLED,
            Permission.ENROLLMENT_REQUEST_ACCEPT_REJECT
    )),

    STUDENT(EnumSet.of(
            Permission.STUDENT_PROFILE_READ_OWN,
            Permission.COURSE_OFFERING_READ,
            Permission.ENROLLMENT_REQUEST_CREATE
    ));

    private final Set<Permission> permissions;

    Role(Set<Permission> permissions) {
        this.permissions = Collections.unmodifiableSet(permissions);
    }

    public long bitmask() {
        long result = 0;
        for(Permission perm : permissions) {
            result |= perm.getMask();
        }
        return result;
    }

//    private final Set<Permission> permissions;
//
//    ADMIN(EnumSet.of(Permission.STAFF_CREATE)),
//
//    INSPECTOR(EnumSet.of(
//            Permission.STUDENT_CREATE,
//            Permission.STUDENT_PROFILE_READ_ANY,
//            Permission.COURSE_CREATE_MODIFY,
//            Permission.COURSE_OFFERING_READ,
//            Permission.COURSE_OFFERING_CREATE_MODIFY
//    )),
//
//    TEACHER(EnumSet.of(
//            Permission.COURSE_OFFERING_READ,
//            Permission.STUDENT_PROFILE_READ_ENROLLED,
//            Permission.ENROLLMENT_REQUEST_ACCEPT_REJECT
//    )),
//
//    STUDENT(EnumSet.of(
//            Permission.STUDENT_PROFILE_READ_OWN,
//            Permission.COURSE_OFFERING_READ,
//            Permission.ENROLLMENT_REQUEST_CREATE
//    ));
//
//    Role(EnumSet<Permission> permissions) {
//        this.permissions = Collections.unmodifiableSet(permissions);
//    }
//
//    public Set<Permission> getPermissions() {
//        return this.permissions;
//    }



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
