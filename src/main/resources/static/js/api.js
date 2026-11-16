// Shared by every page: all calls to the backend go through here.
//
// Backend contract:
//   POST  /api/v1/auth/login                 {email, password} -> {accessToken, tokenType, expiresIn} | 401
//   GET   /api/v1/users/me                   -> {id, email, firstName, lastName, role,
//                                                 student: {facultyNumber, specialty, studyYear, status} | null}
//                                                (CurrentUserController; on a 404 the profile shows
//                                                 only what the token holds: user id and role)
//   GET   /api/v1/offerings?open=true        -> [{id, courseCode, courseTitle, semester, teacherName,
//                                                  capacity, enrolledCount, availableSeats, isOpen}]
//   POST  /api/v1/enrollments                {studentId, offeringId} -> 201 | 400 full/closed | 409 already
//   PATCH /api/v1/enrollments/{id}/cancel
//   GET   /api/v1/students/{id}/enrollments  -> [{id, offeringId, courseCode, courseTitle, semester, status}]
//
// Every request after login sends "Authorization: Bearer <accessToken>".
//
// Demo mode without a backend: open /login.html?mock=1 (any email; password "wrong" fails).
// /login.html?mock=0 turns it off again.
const Api = (() => {
    const URLS = {
        login: '/api/v1/auth/login',
        me: '/api/v1/users/me',
        offerings: '/api/v1/offerings?open=true',
        enroll: '/api/v1/enrollments',
        cancel: (id) => `/api/v1/enrollments/${encodeURIComponent(id)}/cancel`,
        studentEnrollments: (id) => `/api/v1/students/${encodeURIComponent(id)}/enrollments`,
    };

    const TOKEN_KEY = 'unicloud.token';
    const MOCK_KEY = 'unicloud.mock';
    const MOCK_DATA_KEY = 'unicloud.mockData';

    const mockParam = new URLSearchParams(location.search).get('mock');
    if (mockParam === '1') sessionStorage.setItem(MOCK_KEY, '1');
    if (mockParam === '0') {
        sessionStorage.removeItem(MOCK_KEY);
        sessionStorage.removeItem(MOCK_DATA_KEY);
        sessionStorage.removeItem(TOKEN_KEY);
    }
    const mock = sessionStorage.getItem(MOCK_KEY) === '1';

    // Server messages are English; show them in Bulgarian.
    const MESSAGES = {
        'Invalid email or password': 'Грешен имейл или парола.',
        'Course is closed for enrollment': 'Дисциплината е затворена за записване.',
        'Course is full, no available seats': 'Няма свободни места в дисциплината.',
        'Student is already enrolled in this course': 'Вече сте записани в тази дисциплина.',
        'Only active students can enroll': 'Само активни студенти могат да се записват.',
        'Enrollment not found': 'Записването не е намерено.',
        'Enrollment is already cancelled': 'Записването вече е отказано.',
        'You do not have permission to perform this action': 'Нямате права за това действие.',
    };

    class ApiError extends Error {
        constructor(status, message) {
            super(MESSAGES[message] || message);
            this.status = status;
        }
    }

    // ---- token ----

    function token() {
        return sessionStorage.getItem(TOKEN_KEY);
    }

    function base64UrlToJson(part) {
        const base64 = part.replace(/-/g, '+').replace(/_/g, '/');
        const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
        return JSON.parse(new TextDecoder().decode(bytes));
    }

    // The JWT payload: sub = user id, role = STUDENT | TEACHER | INSPECTOR | ADMIN, exp = expiry.
    function claims() {
        const t = token();
        if (!t) return null;
        try {
            return base64UrlToJson(t.split('.')[1]);
        } catch (e) {
            return null;
        }
    }

    function isLoggedIn() {
        const c = claims();
        return !!c && (!c.exp || c.exp * 1000 > Date.now());
    }

    function logout() {
        sessionStorage.removeItem(TOKEN_KEY);
    }

    function currentUserId() {
        const c = claims();
        return c ? Number(c.sub) : null;
    }

    function currentRole() {
        const c = claims();
        return c ? c.role : null;
    }

    // ---- HTTP ----

    async function request(url, options = {}) {
        const headers = { Accept: 'application/json', ...(options.headers || {}) };
        if (token()) headers.Authorization = `Bearer ${token()}`;
        if (options.body) headers['Content-Type'] = 'application/json';

        let response;
        try {
            response = await fetch(url, { ...options, headers });
        } catch (e) {
            throw new ApiError(0, 'Няма връзка със сървъра.');
        }

        const text = await response.text();
        let body = null;
        try { body = text ? JSON.parse(text) : null; } catch (e) { /* not JSON */ }

        if (!response.ok) {
            // An expired or rejected token: back to the login page.
            if (response.status === 401 && url !== URLS.login) {
                logout();
                location.replace('/login.html');
            }
            const message = (body && (body.message || body.error)) || `Грешка ${response.status}`;
            throw new ApiError(response.status, message);
        }
        return body;
    }

    // ---- API ----

    async function login(email, password) {
        if (mock) return Mock.login(email, password);
        const body = await request(URLS.login, {
            method: 'POST',
            body: JSON.stringify({ email, password }),
        });
        if (!body || !body.accessToken) throw new ApiError(500, 'Сървърът не върна токен.');
        sessionStorage.setItem(TOKEN_KEY, body.accessToken);
    }

    // The full profile. Until the backend has /users/me, falls back to what the token holds.
    async function me() {
        if (mock) return Mock.me();
        try {
            return await request(URLS.me);
        } catch (e) {
            if (e.status !== 404) throw e;
            return { id: currentUserId(), role: currentRole(), partial: true };
        }
    }

    function offerings() {
        return mock ? Mock.offerings() : request(URLS.offerings);
    }

    function myEnrollments() {
        return mock ? Mock.enrollments() : request(URLS.studentEnrollments(currentUserId()));
    }

    function enroll(offeringId) {
        if (mock) return Mock.enroll(offeringId);
        return request(URLS.enroll, {
            method: 'POST',
            body: JSON.stringify({ studentId: currentUserId(), offeringId }),
        });
    }

    function cancel(enrollmentId) {
        return mock ? Mock.cancel(enrollmentId) : request(URLS.cancel(enrollmentId), { method: 'PATCH' });
    }

    // ---- demo mode: a fake backend with the same answers as the real one ----

    const Mock = (() => {
        const USER = {
            id: 4, email: 'student1@student.tu-sofia.bg', firstName: 'Александър', lastName: 'Стоянов',
            role: 'STUDENT',
            student: { facultyNumber: '121219001', specialty: 'Компютърни науки', studyYear: 2, status: 'ACTIVE' },
        };
        const SEED = {
            nextId: 100,
            offerings: [
                { id: 1, courseCode: 'CS101', courseTitle: 'Въведение в програмирането', teacherName: 'Иван Петров', capacity: 30, taken: 2 },
                { id: 2, courseCode: 'CS201', courseTitle: 'Структури от данни и алгоритми', teacherName: 'Иван Петров', capacity: 25, taken: 1 },
                { id: 3, courseCode: 'CS301', courseTitle: 'Бази данни', teacherName: 'Мария Иванова', capacity: 20, taken: 1 },
                { id: 4, courseCode: 'CS302', courseTitle: 'Уеб технологии', teacherName: 'Мария Иванова', capacity: 25, taken: 0 },
                { id: 6, courseCode: 'CS402', courseTitle: 'Разпределени системи', teacherName: 'Георги Димитров', capacity: 3, taken: 3 },
                { id: 8, courseCode: 'MATH201', courseTitle: 'Дискретна математика', teacherName: 'Мария Иванова', capacity: 35, taken: 0 },
            ],
            enrollments: [
                { id: 1, offeringId: 1, status: 'ENROLLED' },
                { id: 2, offeringId: 2, status: 'ENROLLED' },
            ],
        };

        function load() {
            try {
                return JSON.parse(sessionStorage.getItem(MOCK_DATA_KEY)) || structuredClone(SEED);
            } catch (e) {
                return structuredClone(SEED);
            }
        }

        function save(data) {
            sessionStorage.setItem(MOCK_DATA_KEY, JSON.stringify(data));
        }

        function fail(status, message) {
            return Promise.reject(new ApiError(status, message));
        }

        function toOffering(o) {
            return {
                id: o.id, courseCode: o.courseCode, courseTitle: o.courseTitle, semester: '2026-27-W',
                teacherName: o.teacherName, capacity: o.capacity, enrolledCount: o.taken,
                availableSeats: o.capacity - o.taken, isOpen: true,
            };
        }

        function toEnrollment(data, e) {
            const o = data.offerings.find((x) => x.id === e.offeringId);
            return {
                id: e.id, studentId: USER.id, offeringId: o.id, courseCode: o.courseCode,
                courseTitle: o.courseTitle, semester: '2026-27-W', status: e.status,
            };
        }

        function base64Url(object) {
            const bytes = new TextEncoder().encode(JSON.stringify(object));
            return btoa(String.fromCharCode(...bytes)).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
        }

        return {
            login(email, password) {
                if (password === 'wrong') return fail(401, 'Invalid email or password');
                const payload = { sub: String(USER.id), role: USER.role, exp: Math.floor(Date.now() / 1000) + 1800 };
                sessionStorage.setItem(TOKEN_KEY, `${base64Url({ alg: 'none' })}.${base64Url(payload)}.mock`);
                return Promise.resolve();
            },
            me: () => Promise.resolve(USER),
            offerings: () => Promise.resolve(load().offerings.map(toOffering)),
            enrollments() {
                const data = load();
                return Promise.resolve(data.enrollments.map((e) => toEnrollment(data, e)));
            },
            enroll(offeringId) {
                const data = load();
                const o = data.offerings.find((x) => x.id === offeringId);
                if (!o) return fail(404, 'CourseOffering not found');
                let e = data.enrollments.find((x) => x.offeringId === offeringId);
                if (e && e.status === 'ENROLLED') return fail(409, 'Student is already enrolled in this course');
                if (o.taken >= o.capacity) return fail(400, 'Course is full, no available seats');
                if (e) e.status = 'ENROLLED';
                else data.enrollments.push(e = { id: data.nextId++, offeringId, status: 'ENROLLED' });
                o.taken++;
                save(data);
                return Promise.resolve(toEnrollment(data, e));
            },
            cancel(enrollmentId) {
                const data = load();
                const e = data.enrollments.find((x) => x.id === enrollmentId);
                if (!e) return fail(404, 'Enrollment not found');
                if (e.status === 'CANCELLED') return fail(409, 'Enrollment is already cancelled');
                e.status = 'CANCELLED';
                data.offerings.find((x) => x.id === e.offeringId).taken--;
                save(data);
                return Promise.resolve(toEnrollment(data, e));
            },
        };
    })();

    document.addEventListener('DOMContentLoaded', () => {
        const banner = document.getElementById('mock-banner');
        if (banner && mock) banner.hidden = false;
    });

    return { login, logout, isLoggedIn, currentRole, me, offerings, myEnrollments, enroll, cancel, ApiError };
})();

// Shared page helpers.
const Ui = {
    ROLES: { STUDENT: 'Студент', TEACHER: 'Преподавател', INSPECTOR: 'Инспектор', ADMIN: 'Администратор' },
    STATUSES: { ACTIVE: 'Активен', INTERRUPTED: 'Прекъснал', GRADUATED: 'Завършил' },

    // For pages behind login: redirect if not logged in, wire up the logout button.
    requireLogin() {
        const logout = document.getElementById('logout');
        if (logout) {
            logout.addEventListener('click', () => {
                Api.logout();
                location.replace('/login.html');
            });
        }
        if (!Api.isLoggedIn()) {
            Api.logout();
            location.replace('/login.html');
            return false;
        }
        return true;
    },

    cell(text) {
        const td = document.createElement('td');
        td.textContent = text == null ? '' : text;
        return td;
    },
};
