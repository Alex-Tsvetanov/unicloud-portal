package bg.tusofia.unicloud.portal.student.service;

import bg.tusofia.unicloud.portal.student.domain.Student;
import bg.tusofia.unicloud.portal.student.domain.StudentRepository;
import bg.tusofia.unicloud.portal.student.domain.StudentStatus;
import bg.tusofia.unicloud.portal.user.domain.Role;
import bg.tusofia.unicloud.portal.user.domain.User;
import bg.tusofia.unicloud.portal.user.service.RegisterUserCommand;
import bg.tusofia.unicloud.portal.user.service.UserRegistrationService;
import org.springframework.stereotype.Service;

@Service
public class StudentRegistrationService {

    private final UserRegistrationService userRegistrationSvc;
    private final StudentRepository studentRepository;

    public StudentRegistrationService(UserRegistrationService userRegistrationSvc,
                                      StudentRepository studentRepository) {
        this.userRegistrationSvc = userRegistrationSvc;
        this.studentRepository = studentRepository;
    }

    public Student registerStudent(String email,
                                   String password,
                                   String firstName,
                                   String lastName,
                                   String facultyNumber,
                                   String specialty,
                                   int studyYear,
                                   String studentStatus,
                                   String principalEmail) {


        User studentUser = userRegistrationSvc.register(
                new RegisterUserCommand(
                        email,
                        password,
                        firstName,
                        lastName,
                        Role.STUDENT.toString()), principalEmail);

        Student newStudent = Student.createNew(studentUser.getId(), facultyNumber, specialty, studyYear);

        newStudent = studentRepository.save(newStudent);

        return newStudent;

    }
}
