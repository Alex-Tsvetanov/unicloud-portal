(() => {
    if (!Ui.requireLogin()) return;

    const $ = (id) => document.getElementById(id);

    function showError(message) {
        $('loading').hidden = true;
        $('error').textContent = message;
        $('error').hidden = false;
    }

    async function loadEnrollments() {
        let list;
        try {
            list = await Api.myEnrollments();
        } catch (e) {
            return; // the profile still works without this section
        }
        const active = (list || []).filter((item) => item.status === 'ENROLLED');
        for (const item of active) {
            const tr = document.createElement('tr');
            tr.append(Ui.cell(item.courseCode), Ui.cell(item.courseTitle), Ui.cell(item.semester));
            $('enrollment-rows').append(tr);
        }
        $('no-enrollments').hidden = active.length > 0;
        $('enrollments').hidden = false;
    }

    async function load() {
        let user;
        try {
            user = await Api.me();
        } catch (e) {
            showError(e.status === 0 ? e.message : 'Профилът не може да бъде зареден.');
            return;
        }

        $('role').textContent = Ui.ROLES[user.role] || user.role || '';

        if (user.partial) {
            $('full-name').textContent = `Потребител №${user.id}`;
            $('partial').hidden = false;
        } else {
            $('full-name').textContent = `${user.firstName} ${user.lastName}`;
            $('email').textContent = user.email;
            $('account').hidden = false;
        }

        if (user.student) {
            $('faculty-number').textContent = user.student.facultyNumber;
            $('specialty').textContent = user.student.specialty;
            $('study-year').textContent = user.student.studyYear;
            $('status').textContent = Ui.STATUSES[user.student.status] || user.student.status;
            $('student').hidden = false;
        }

        if (user.role === 'STUDENT') loadEnrollments();

        $('loading').hidden = true;
        $('profile').hidden = false;
    }

    load();
})();
