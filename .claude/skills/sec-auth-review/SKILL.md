---
name: sec-auth-review
description: Review UniCloud registration, login, session/token handling and password storage - hashing algorithm and cost, password policy, account enumeration, brute-force and credential-stuffing protection, session fixation, CSRF, logout, password reset, MFA. Use for "review login", "review registration", "how are passwords stored", "is authentication secure", or any change to auth code, Spring Security config, user entities or the users table.
---

# Registration, authentication and password storage

Reports findings into the vulnerability register. Read `AGENTS.md` first.

## Where to look

```bash
grep -rnE 'SecurityFilterChain|PasswordEncoder|BCrypt|Argon2|Pbkdf2|DelegatingPasswordEncoder|formLogin|httpBasic|oauth2|jwt|Jwt|session|csrf|rememberMe|AuthenticationManager|UserDetails' src/main/java src/main/resources
grep -rniE 'password|register|signup|login|reset|token' src/main/java --include=*.java -l
grep -n 'password_hash\|role\|enabled' src/main/resources/db/migration/*.sql
grep -n 'security' pom.xml
```

## Checks

### Password storage

- A `PasswordEncoder` bean exists and is used on every write to `password_hash` (registration, reset, admin create). Prefer `PasswordEncoderFactories.createDelegatingPasswordEncoder()` (stores `{bcrypt}` / `{argon2}` prefix, allows upgrades) or BCrypt with strength ≥ 12, or Argon2id with Spring's current defaults or stronger.
- No plain, reversible (AES/Base64) or fast (MD5/SHA-1/SHA-256 without a KDF) hashing. No custom salt handling.
- `password_hash VARCHAR(255)` fits the chosen encoder's output.
- Password never logged, never in a response DTO, never in `toString()` of an entity (check Lombok `@Data`/`@ToString`), never in exception messages.
- Upgrading hashes on login if the encoder changes (`UserDetailsPasswordService`) — hardening.

### Password policy

- Minimum length ≥ 12 (or ≥ 8 with a breached-password check), no forced composition rules, max length ≥ 64 and ≤ 128 (BCrypt only uses 72 bytes: reject or pre-hash longer inputs explicitly).

### Registration

- Role is not client-controlled (STUDENT by default; staff accounts created by ADMIN). Look for `role` in request DTOs and `@RequestBody User`.
- Faculty number / email ownership: how does the portal know the person owns them? (University directory, email verification, or inspector approval before `enabled = true`.) Without it, anyone can claim another student's faculty number first: a finding once registration exists.
- Duplicate email/faculty number errors don't reveal which accounts exist to anonymous users, or the difference is rate-limited.

### Login and sessions

- Generic failure message and the same status and similar timing for "no such user" and "wrong password" (Spring's `DaoAuthenticationProvider` handles timing if used; custom code often doesn't).
- Brute force: per-account and per-IP limits or progressive delay; lockout must not let anyone lock out a student before 08:00 enrollment (prefer delay + CAPTCHA-free throttling over hard lockout). With 2 to 10 replicas, in-memory counters per replica are weak: note it.
- Disabled accounts (`users.enabled = false`) cannot log in; check `UserDetails.isEnabled()` maps to it.
- Sessions: with multiple replicas and no sticky sessions, server-side `HttpSession` breaks or needs a shared store. If JWT is used: signature algorithm fixed (no `none`, no HS/RS confusion), secret from env and ≥ 256 bits, short expiry, `iss`/`aud`/`exp` validated, and a revocation story for role changes and logout.
- Cookies: `Secure`, `HttpOnly`, `SameSite=Lax` or `Strict` (`server.servlet.session.cookie.*`). Session ID rotated on login (Spring default; check it isn't disabled).
- CSRF: enabled for cookie-authenticated state-changing requests. Disabling it is only fine for pure bearer-token APIs.
- Logout invalidates the server session or token.

### Password reset (when it exists)

- Token: random ≥ 128 bits, single-use, expires in ≤ 1 hour, stored hashed, sent only to the verified email; the response is the same whether or not the email exists.

### MFA

- In scope for ADMIN and INSPECTOR when those roles get write power. In week 1 it may be `not applicable yet`; record it in requirements, not as a vulnerability.

## Verify with tests (local only)

When code exists, write or run MockMvc tests against the local Docker MySQL: registration with `"role":"ADMIN"` is ignored or rejected; login responses identical for unknown user vs. wrong password; disabled user can't log in; stored hash starts with `{bcrypt}`/`$2` or `{argon2}`/`$argon2id`; password not present in any response or log line captured in the test.

## Output

Findings in the `sec-sweep` shape (refs: SR-xx, T-xx, CWE-256/257/307/521/916, OWASP ASVS V2/V3).
