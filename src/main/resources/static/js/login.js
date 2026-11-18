(() => {
    // login.html?sso=1: another UniCloud module sent the user here (docs/sso.md). After login the
    // browser goes back to that module instead of the profile.
    const sso = new URLSearchParams(location.search).get('sso') === '1';

    async function continueSso() {
        const next = await Api.ssoSession();
        if (next) {
            location.replace(next);
        } else {
            showError('Времето за вход изтече. Върнете се в приложението и опитайте отново.');
        }
    }

    if (Api.isLoggedIn()) {
        if (!sso) {
            location.replace('/profile.html');
            return;
        }
        // Already logged in to the portal: no password needed. If the token is refused, the form
        // stays and the user logs in again.
        continueSso().catch(() => Api.logout());
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
            if (sso) await continueSso();
            else location.assign('/profile.html');
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
