## ADDED Requirements

### Requirement: No public sign-up
The portal SHALL NOT offer self-service account creation. Inspectors MUST be the only users who create students, and admins MUST be the only users who create staff (teachers, deans, inspectors and admins).

#### Scenario: Anonymous visitor looks for sign-up
- **WHEN** an anonymous visitor opens the sign-in page
- **THEN** the page offers sign-in only
- **AND** no route creates an account without a signed-in creator

#### Scenario: Inspector creates a student
- **WHEN** a user with `STUDENT_CREATE` submits the new-student form with email, names, faculty number, specialty, study year and a first password
- **THEN** the portal creates a user with role `STUDENT` and an academic record
- **AND** the new user's permissions are the `STUDENT` defaults

#### Scenario: Admin creates a staff member
- **WHEN** a user with `STAFF_CREATE` submits the new-staff form with role `TEACHER`, `DEAN`, `INSPECTOR` or `ADMIN`, a faculty when the role has one, and a first password
- **THEN** the portal creates the user with that role's default permissions

#### Scenario: Creator lacks the permission
- **WHEN** a user without `STUDENT_CREATE` posts the new-student form
- **THEN** the portal answers 403 and creates nothing

### Requirement: Unique identity data
The portal MUST keep emails and faculty numbers unique. It SHALL store emails stripped and lower-cased.

#### Scenario: Duplicate email
- **WHEN** a creator submits an email that already belongs to a user, in any letter case
- **THEN** the form shows "this email is already used" in Bulgarian
- **AND** no user is created

#### Scenario: Duplicate faculty number
- **WHEN** an inspector submits a faculty number that already exists
- **THEN** the form shows an error and no user or academic record is created

### Requirement: First password must be changed
A user created through the portal UI MUST change the first password at the first sign-in, before any other page or any SSO login completes. Users created through the staging tools are exempt.

#### Scenario: First sign-in
- **WHEN** a user created by an inspector signs in with the first password
- **THEN** the portal shows the change-password page
- **AND** every other page and `/oauth2/authorize` redirect back to it until the password is changed

#### Scenario: Staging user signs in
- **WHEN** a user created through the staging tools signs in
- **THEN** the portal does not ask for a password change

### Requirement: Password storage
The portal SHALL hash passwords with Argon2id and MUST never store, log or display a password. A password MUST be 8 to 72 characters.

#### Scenario: Stored hash
- **WHEN** a user is created with a password
- **THEN** the database holds an Argon2id hash and not the password

#### Scenario: Too short
- **WHEN** a creator or a user submits a password of 7 characters
- **THEN** the form rejects it with a Bulgarian message

### Requirement: Sign-in with email and password
The portal SHALL sign a user in with email and password and start a cookie session. A wrong email, a wrong password and a disabled user MUST give the same error message.

#### Scenario: Correct credentials
- **WHEN** an enabled user submits the right email and password
- **THEN** the portal sets the session cookie with a new session id
- **AND** redirects to the user's home page, or to the waiting SSO login if there is one

#### Scenario: Wrong credentials
- **WHEN** a visitor submits a wrong password, an unknown email or the email of a disabled user
- **THEN** the page shows "Грешен имейл или парола" and no session is created

### Requirement: One cookie session
The portal UI and single sign-on SHALL share one server-side session, stored in PostgreSQL, in the cookie `__Host-UNICLOUD_SSO` with `Path=/`, no `Domain`, `HttpOnly`, `Secure` and `SameSite=Lax`. The session MUST end after 30 minutes without a request, and 8 hours after the password was typed, whichever comes first. The portal MUST NOT issue its own JWT and MUST NOT serve `/api/v1/auth/login` or `/api/v1/auth/sso-session`.

#### Scenario: Idle timeout
- **WHEN** a signed-in user sends no request for 31 minutes
- **THEN** the next page asks for the password again

#### Scenario: Maximum age
- **WHEN** a user keeps using the portal for 8 hours and 1 minute after typing the password
- **THEN** the session ends and the next request is anonymous

#### Scenario: Exactly at the limit
- **WHEN** a request arrives exactly 8 hours after the password was typed
- **THEN** the session is still valid

#### Scenario: Another replica
- **WHEN** a user's next request reaches a different replica
- **THEN** the user is still signed in

#### Scenario: Old token routes are gone
- **WHEN** a client calls `POST /api/v1/auth/login`
- **THEN** the portal answers 404

### Requirement: Sign-out
Signing out SHALL end the session for the whole browser, which also ends the SSO session.

#### Scenario: Sign out
- **WHEN** a signed-in user presses "Изход"
- **THEN** the session is deleted and the cookie is cleared
- **AND** the next `/oauth2/authorize` from that browser goes to the sign-in page

### Requirement: CSRF protection
Every request that changes state through the portal UI MUST carry a CSRF token bound to the session. Requests without a valid token MUST be refused with 403.

#### Scenario: Missing token
- **WHEN** a form post arrives with the session cookie but without the CSRF token
- **THEN** the portal answers 403 and changes nothing

#### Scenario: htmx request
- **WHEN** htmx sends a state-changing request with the token in a header
- **THEN** the portal accepts it

### Requirement: Disabled users
A disabled user SHALL NOT sign in, and an existing session of a disabled user MUST stop working at the next request.

#### Scenario: Disabled during a session
- **WHEN** an admin disables a user who is signed in
- **THEN** that user's next request is treated as anonymous
