package bg.tusofia.unicloud.portal.exception;

public class EnrollmentException extends RuntimeException {

    private final ErrorCode errorCode;

    public enum ErrorCode {
        COURSE_CLOSED("Course is closed for enrollment"),
        COURSE_FULL("Course is full, no available seats"),
        ALREADY_ENROLLED("Student is already enrolled in this course"),
        STUDENT_NOT_ACTIVE("Only active students can enroll"),
        ENROLLMENT_NOT_FOUND("Enrollment not found"),
        ALREADY_CANCELLED("Enrollment is already cancelled");

        private final String defaultMessage;

        ErrorCode(String defaultMessage) {
            this.defaultMessage = defaultMessage;
        }

        public String getDefaultMessage() {
            return defaultMessage;
        }
    }

    public EnrollmentException(ErrorCode errorCode) {
        super(errorCode.getDefaultMessage());
        this.errorCode = errorCode;
    }

    public EnrollmentException(ErrorCode errorCode, String message) {
        super(message);
        this.errorCode = errorCode;
    }

    public ErrorCode getErrorCode() {
        return errorCode;
    }
}
