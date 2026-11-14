package bg.tusofia.unicloud.portal.user.domain;

import java.util.Arrays;
import java.util.EnumSet;
import java.util.Set;
import java.util.stream.Collectors;

public enum Permission {

    STAFF_CREATE(0),

    STUDENT_CREATE(1),
    STUDENT_PROFILE_READ_OWN(2),
    STUDENT_PROFILE_READ_ANY(3),
    STUDENT_PROFILE_READ_ENROLLED(4),

    COURSE_CREATE_MODIFY(5),
    COURSE_OFFERING_READ(6),
    COURSE_OFFERING_CREATE_MODIFY(7),

    ENROLLMENT_REQUEST_CREATE(8),
    ENROLLMENT_REQUEST_ACCEPT_REJECT(9);


    private final int bitPosition;

    Permission(int bitPosition) {
        this.bitPosition = bitPosition;
    }

    public long getMask() {
        return 1L << bitPosition;
    }

    public boolean isGranted(long mask) {
        return (mask & getMask()) != 0;
    }

    public static Set<Permission> fromMask(long mask) {
        return Arrays.stream(Permission.values())
                .filter(permission -> permission.isGranted(mask))
                .collect(Collectors.toCollection(
                        () -> EnumSet.noneOf(Permission.class)
                ));
    }
}

