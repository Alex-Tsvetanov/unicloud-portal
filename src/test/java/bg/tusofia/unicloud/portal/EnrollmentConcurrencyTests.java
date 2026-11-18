package bg.tusofia.unicloud.portal;

import bg.tusofia.unicloud.portal.dto.EnrollmentRequest;
import bg.tusofia.unicloud.portal.entity.Course;
import bg.tusofia.unicloud.portal.entity.CourseOffering;
import bg.tusofia.unicloud.portal.exception.EnrollmentException;
import bg.tusofia.unicloud.portal.repository.CourseOfferingRepository;
import bg.tusofia.unicloud.portal.repository.CourseRepository;
import bg.tusofia.unicloud.portal.repository.EnrollmentRepository;
import bg.tusofia.unicloud.portal.repository.UserRepository;
import bg.tusofia.unicloud.portal.service.EnrollmentService;
import bg.tusofia.unicloud.portal.support.TestJwtKeys;
import org.junit.jupiter.api.AfterEach;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.RepeatedTest;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.test.context.SpringBootTest;
import org.springframework.test.context.DynamicPropertyRegistry;
import org.springframework.test.context.DynamicPropertySource;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Students racing for the last seats must never oversell an offering, on MySQL (production) or
 * PostgreSQL (staging). The pipeline runs this class on both. Each enrollment commits for real,
 * so the test removes its own rows afterwards.
 */
@SpringBootTest
class EnrollmentConcurrencyTests {

	/** The active students in V2__test_data.sql. */
	private static final List<Long> ACTIVE_STUDENT_IDS = List.of(4L, 5L, 6L, 8L);

	/** ivan.petrov@tu-sofia.bg in V2__test_data.sql. */
	private static final long TEACHER_ID = 1L;

	private static final int CAPACITY = 2;

	@DynamicPropertySource
	static void jwtKeys(DynamicPropertyRegistry registry) {
		TestJwtKeys.register(registry);
	}

	@Autowired
	private EnrollmentService enrollmentService;

	@Autowired
	private CourseRepository courseRepository;

	@Autowired
	private CourseOfferingRepository offeringRepository;

	@Autowired
	private EnrollmentRepository enrollmentRepository;

	@Autowired
	private UserRepository userRepository;

	private Course course;
	private CourseOffering offering;

	@BeforeEach
	void createOffering() {
		// Codes are at most 16 characters and must be unique across repetitions
		String code = "RACE" + System.nanoTime() % 1_000_000_000L;
		course = courseRepository.save(new Course(code, "Concurrency test", 1));
		offering = offeringRepository.save(new CourseOffering(
				course, "2099-00-T", userRepository.findById(TEACHER_ID).orElseThrow(), CAPACITY));
	}

	@AfterEach
	void removeOffering() {
		enrollmentRepository.deleteAll(enrollmentRepository.findByOfferingId(offering.getId()));
		offeringRepository.delete(offering);
		courseRepository.delete(course);
	}

	@RepeatedTest(5)
	void studentsRacingForTheLastSeatsNeverOversell() throws Exception {
		CountDownLatch start = new CountDownLatch(1);
		ExecutorService pool = Executors.newFixedThreadPool(ACTIVE_STUDENT_IDS.size());
		List<Future<Boolean>> results = new ArrayList<>();
		try {
			for (Long studentId : ACTIVE_STUDENT_IDS) {
				results.add(pool.submit(() -> {
					start.await();
					try {
						enrollmentService.enroll(new EnrollmentRequest(studentId, offering.getId()));
						return true;
					} catch (EnrollmentException e) {
						assertEquals(EnrollmentException.ErrorCode.COURSE_FULL, e.getErrorCode());
						return false;
					}
				}));
			}
			start.countDown();

			int enrolled = 0;
			for (Future<Boolean> result : results) {
				if (result.get(30, TimeUnit.SECONDS)) {
					enrolled++;
				}
			}

			assertEquals(CAPACITY, enrolled, "successful enrollments");
			assertEquals(CAPACITY, enrollmentRepository.countEnrolledByOfferingId(offering.getId()),
					"ENROLLED rows in the database");
		} finally {
			pool.shutdownNow();
		}
	}
}
