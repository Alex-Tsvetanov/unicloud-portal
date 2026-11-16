(() => {
    if (Api.isLoggedIn()) {
        location.replace('/profile.html');
        return;
    }

    const form = document.getElementById('login-form');
    const email = document.getElementById('email');
    const password = document.getElementById('password');
    const error = document.getElementById('error');
    const submit = document.getElementById('submit');

    function showError(message) {
        error.textContent = message;
        error.hidden = false;
    }

    form.addEventListener('submit', async (event) => {
        event.preventDefault();
        error.hidden = true;

        if (!email.value.trim() || !password.value) {
            showError('Въведете имейл и парола.');
            return;
        }
        if (!email.checkValidity()) {
            showError('Невалиден имейл адрес.');
            return;
        }

        submit.disabled = true;
        try {
            await Api.login(email.value.trim(), password.value);
            location.assign('/profile.html');
        } catch (e) {
            if (e.status === 401 || e.status === 400) showError('Грешен имейл или парола.');
            else if (e.status === 404) showError('Входът още не е наличен на сървъра.');
            else if (e.status === 0) showError(e.message);
            else showError('Възникна грешка. Опитайте отново.');
            password.value = '';
            password.focus();
        } finally {
            submit.disabled = false;
        }
    });
})();
