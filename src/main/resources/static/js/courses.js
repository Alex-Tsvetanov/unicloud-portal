(() => {
    if (!Ui.requireLogin()) return;

    const $ = (id) => document.getElementById(id);
    const isStudent = Api.currentRole() === 'STUDENT';

    function showMessage(text, kind) {
        $('message').textContent = text;
        $('message').className = `message message-${kind}`;
        $('message').hidden = false;
    }

    function button(label, className, onClick) {
        const b = document.createElement('button');
        b.type = 'button';
        b.className = className;
        b.textContent = label;
        b.addEventListener('click', () => {
            b.disabled = true;
            onClick();
        });
        return b;
    }

    // The enroll button stays active on full courses: the server decides and the page shows its answer.
    async function enroll(offering) {
        try {
            await Api.enroll(offering.id);
            showMessage(`Записахте се успешно в ${offering.courseCode} · ${offering.courseTitle}.`, 'success');
        } catch (e) {
            showMessage(`${offering.courseCode}: ${e.message}`, 'error');
        }
        load();
    }

    async function cancel(offering, enrollment) {
        try {
            await Api.cancel(enrollment.id);
            showMessage(`Отказахте се от ${offering.courseCode} · ${offering.courseTitle}.`, 'success');
        } catch (e) {
            showMessage(`${offering.courseCode}: ${e.message}`, 'error');
        }
        load();
    }

    function withClass(td, className) {
        td.className = className;
        return td;
    }

    function seatsCell(o) {
        const td = Ui.cell(`${o.enrolledCount} / ${o.capacity}`);
        if (o.availableSeats <= 0) {
            const badge = document.createElement('span');
            badge.className = 'badge badge-full';
            badge.textContent = 'Пълна';
            td.append(' ', badge);
        }
        return td;
    }

    function actionCell(o, enrollment) {
        const td = document.createElement('td');
        td.className = 'actions-col';
        if (!isStudent) return td;
        if (enrollment) {
            const done = document.createElement('span');
            done.className = 'enrolled';
            done.textContent = 'Записан';
            td.append(done, button('Откажи се', 'btn btn-secondary', () => cancel(o, enrollment)));
        } else {
            td.append(button('Запиши се', 'btn', () => enroll(o)));
        }
        return td;
    }

    async function load() {
        let offerings;
        let enrollments = [];
        try {
            [offerings, enrollments] = await Promise.all([
                Api.offerings(),
                isStudent ? Api.myEnrollments() : Promise.resolve([]),
            ]);
        } catch (e) {
            $('loading').hidden = true;
            showMessage(e.status === 0 ? e.message : 'Дисциплините не могат да бъдат заредени.', 'error');
            return;
        }

        const enrolledBy = new Map(
            (enrollments || []).filter((e) => e.status === 'ENROLLED').map((e) => [e.offeringId, e]));

        const rows = $('rows');
        rows.replaceChildren();
        for (const o of offerings || []) {
            const tr = document.createElement('tr');
            tr.append(
                withClass(Ui.cell(o.courseCode), 'col-code'),
                Ui.cell(o.courseTitle),
                withClass(Ui.cell(o.teacherName), 'col-teacher'),
                seatsCell(o),
                actionCell(o, enrolledBy.get(o.id)),
            );
            rows.append(tr);
        }

        $('loading').hidden = true;
        $('table').hidden = !offerings || offerings.length === 0;
        $('empty').hidden = !$('table').hidden;
    }

    load();
})();
