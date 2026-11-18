# Single sign-on through the portal

The other UniCloud modules log users in through the portal. A user signs in once, at the portal, and every module trusts that sign-in.

The portal is a standard **OpenID Connect** provider. It uses the **OAuth 2.0 authorization code flow with PKCE**. Every module can use an existing OIDC client library in its own language. Nothing here is specific to UniCloud except the user claims.

## Contents

1. [The rules in short](#the-rules-in-short)
2. [Callback URLs per environment](#callback-urls-per-environment)
3. [The flow](#the-flow)
4. [Endpoints](#endpoints)
5. [What a module must build](#what-a-module-must-build)
6. [User claims](#user-claims)
7. [Logout](#logout)
8. [Testing a module locally](#testing-a-module-locally)
9. [Registering a module (portal admins)](#registering-a-module-portal-admins)
10. [How the portal does it](#how-the-portal-does-it)
11. [Not done yet](#not-done-yet)

## The rules in short

- A module sends the browser to the portal's `/oauth2/authorize` with a `redirect_uri` (the "callback"). The portal sends the user back to it after login.
- In production every callback URL is registered in advance, on a host the portal allows. The portal refuses any other URL and does not redirect to it. A matching host or prefix is not enough: the whole URL must match, character for character. Staging and the local mock accept any callback (see [Callback URLs per environment](#callback-urls-per-environment)).
- The callback **never** receives user data or a token. It receives a short-lived, single-use `code`.
- The module's backend exchanges that code for the user's identity by calling the portal directly, server to server.

Why: anything put in a redirect URL ends up in browser history, server logs and `Referer` headers. A callback URL that is not checked lets an attacker send a victim to the login page with their own callback and receive the victim's identity.

## Callback URLs per environment

The callbacks are the `redirect_uri` of a login and the `post_logout_redirect_uri` of a logout.

| Environment | Which callbacks work | Set by |
|---|---|---|
| **Production** | Only URLs registered for the module, matched exactly, `https`, on a host listed in [`sso-callback-hosts.properties`](../src/main/resources/sso-callback-hosts.properties) | The default (`app.sso.callback-policy=allowlist`) |
| **Staging** | Any absolute `http` or `https` URL, registered or not | `APP_SSO_CALLBACK_POLICY=any` in Render |
| **Local mock** | Any absolute `http` or `https` URL, registered or not | The `mock` profile (see [Testing a module locally](#testing-a-module-locally)) |

**The production host list.** `src/main/resources/sso-callback-hosts.properties` holds one comma-separated list. An entry is an exact host (`library.tu-sofia.bg`) or every subdomain of a domain (`*.tu-sofia.bg`, which does not include `tu-sofia.bg` itself). Change it with a pull request.
- A module can register in production only once its host is on the list.
- Taking a host off the list switches off the logins and logouts of every module that uses it, at the next deploy, even if the module stays registered.
- In an emergency, the variable `APP_SSO_ALLOWED_CALLBACK_HOSTS` overrides the whole list without a release.
- The list is empty today, because no module is live yet.

Even staging and the mock accept only `http` and `https` URLs. A `javascript:` or relative URL is refused everywhere.

## The flow

```mermaid
sequenceDiagram
    participant B as Browser
    participant M as Module X (backend)
    participant P as Portal
    B->>M: Open a page, no session
    M->>B: 302 to Portal /oauth2/authorize (client_id, redirect_uri, state, PKCE challenge, nonce)
    B->>P: GET /oauth2/authorize
    P->>P: Check client_id and exact redirect_uri
    P->>B: Login page (only if no portal SSO session)
    B->>P: Email and password
    P->>B: 302 to redirect_uri?code=...&state=...
    B->>M: GET /auth/callback?code=...&state=...
    M->>P: POST /oauth2/token (code, client secret, PKCE verifier)
    P->>M: id_token and access_token
    M->>B: Module session cookie, 302 to a local page
```

If the user already has a portal SSO session, step 4 is skipped: the portal sends the code straight back. The session ends after 30 minutes without use, 8 hours after the user typed the password however busy it is, or when the user logs out of the portal.

## Endpoints

`<portal>` is the portal's public address, for example `https://portal.example`. A library that reads the discovery document needs only that address.

| Endpoint | Purpose |
|---|---|
| `GET /.well-known/openid-configuration` | The discovery document. It lists every endpoint below. |
| `GET /oauth2/authorize` | Starts a login. The browser goes here. |
| `POST /oauth2/token` | Exchanges a code for tokens. The module's backend calls it. |
| `GET /oauth2/jwks` | The public keys that sign the tokens, each with a `kid`. |
| `GET /userinfo` | The user's claims, for a module's access token. |
| `GET /connect/logout` | Ends the portal's SSO session. See [Logout](#logout). |

## What a module must build

The modules are server-side apps. Each one is a **confidential client**: it holds a client secret on its server.

### Start the login

Before redirecting to the portal, the module:

1. Makes a random `state`, a random `nonce` and a random PKCE `code_verifier` (43 to 128 characters from `A-Z a-z 0-9 - . _ ~`).
2. Stores all three in the user's session on the module's side.
3. Redirects the browser to:

```
GET <portal>/oauth2/authorize
    ?response_type=code
    &client_id=<client_id>
    &redirect_uri=<a registered callback URL>
    &scope=openid profile email unicloud
    &state=<state>
    &nonce=<nonce>
    &code_challenge=<BASE64URL(SHA256(code_verifier))>
    &code_challenge_method=S256
```

`scope` decides which user claims the module receives (see [User claims](#user-claims)). Ask only for what the module needs. `openid` is always required.

PKCE is required. Without `code_challenge` the portal answers the callback with `error=invalid_request`.

### The callback endpoint

The callback must accept exactly these two forms:

```
GET <registered redirect_uri>?code=<opaque string>&state=<the state the module sent>
GET <registered redirect_uri>?error=<access_denied|invalid_request|invalid_scope|...>&error_description=<text>&state=<the state the module sent>
```

The handler must:

1. Compare `state` with the value stored in the session. If they differ, reject the request. This stops login CSRF, where an attacker logs the victim into the attacker's account.
2. On `error`, show a message. Do not loop back to the portal automatically.
3. On `code`, exchange it as described below.
4. Validate the `id_token` (see below).
5. Create the module's **own** session, as a cookie with `HttpOnly; Secure; SameSite=Lax`.
6. Redirect the user to a **local path** only. If the module remembers a "return to" page, it must be a relative path that the module stored itself. It must never be an absolute URL taken from the query string.

### Exchanging the code

The module's backend makes a normal HTTPS request to the portal. The browser takes no part in it.

```
POST <portal>/oauth2/token
Authorization: Basic base64(<client_id>:<client_secret>)
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=<code from the callback>
&redirect_uri=<the same redirect_uri as in the login>
&code_verifier=<the PKCE code_verifier>
```

Response:

```json
{
  "access_token": "<JWT>",
  "id_token": "<JWT>",
  "scope": "openid profile email unicloud",
  "token_type": "Bearer",
  "expires_in": 1799
}
```

There is no refresh token. When the module's own session ends, it sends the user through the login again. If the portal's SSO session is still alive, that takes no password.

The endpoint is public, and that is safe. No single value in the request is enough to get a token:

| Value | What it proves |
|---|---|
| `client_secret` | The caller is the module's server. Only that server has the secret. |
| `code` | A user just logged in for this module. The code is single use and expires after 60 seconds. |
| `redirect_uri` | The code is used where it was issued for. |
| `code_verifier` | The caller is the party that started this login. It must hash to the `code_challenge`. |

An attacker who sees a code, in browser history or a log for example, has neither the secret nor the verifier. They cannot exchange it.

A code is exchanged once. A second exchange fails with `invalid_grant`, and the portal also **revokes the tokens the first exchange gave**, because one of the two callers was not the module. This holds even when two portal replicas receive the same code at the same moment.

Errors follow RFC 6749: `400` with `{"error": "invalid_grant"}` for a bad, used or expired code or verifier, and `401` with `{"error": "invalid_client"}` for a wrong client id or secret.

### Validating the `id_token`

Check all of these. Most OIDC libraries do it for you:

- The RS256 signature checks out against the keys at `/oauth2/jwks`. Cache the keys. Fetch them again when a token has a `kid` you do not know.
- `iss` is the portal's address, exactly as in the discovery document.
- `aud` contains the module's own `client_id`.
- `exp` has not passed. Allow a small clock skew, for example 60 seconds.
- `nonce` matches the value stored in the session.

### Rules for every module

- Keep `client_secret` on the server. It never goes into frontend code or into the repo.
- Use PKCE (S256), `state` and `nonce` on every login.
- Exchange the code from the backend only, over HTTPS.
- Never pass portal tokens to browser JavaScript. Never put them in URLs or logs.
- Use your own session for your own app. Use the `role` and `permissions` claims as input to your own authorization checks, not as a replacement for them.
- A module's tokens work only for that module and for `/userinfo`. The portal's own API (`/api/v1/...`) refuses them.

## User claims

Every module may ask for everything the portal knows about the user. Each group of claims comes only with its scope, so a module gets just what it asked for. The list will shrink once each team says what it needs.

The claims are in the `id_token`, and `/userinfo` returns the same ones for the module's access token. They are read from the database at the moment the token is issued.

| Claim | Scope | Example | Notes |
|---|---|---|---|
| `sub` | `openid` | `"4"` | The portal's user id. Stable. Use it as the key, not the email. |
| `auth_time` | `openid` | `1795031651` | When the user typed the password |
| `email` | `email` | `"student1@student.tu-sofia.bg"` | |
| `given_name` | `profile` | `"Александър"` | |
| `family_name` | `profile` | `"Стоянов"` | |
| `name` | `profile` | `"Александър Стоянов"` | |
| `role` | `unicloud` | `"STUDENT"` | One of `STUDENT`, `TEACHER`, `INSPECTOR`, `ADMIN` |
| `permissions` | `unicloud` | `["STUDENT_PROFILE_READ_OWN", "COURSE_OFFERING_READ", "ENROLLMENT_REQUEST_CREATE"]` | The portal's permission names |
| `student` | `unicloud` | `{"faculty_number": "121219001", "specialty": "Компютърни науки", "study_year": 2, "status": "ACTIVE"}` | Only for students. `status` is `ACTIVE`, `INTERRUPTED` or `GRADUATED` |

The ID token travels again in the logout URL (`id_token_hint`), where proxies may log it. That is one more reason to ask only for the scopes you use.

A user who is disabled or deleted gets no new tokens: the exchange fails with `invalid_grant`.

## Logout

**Module only.** The module ends its own session. The user stays logged in at the portal, so the next module login takes no password.

**Everywhere.** The module ends its own session, then redirects the browser to:

```
GET <portal>/connect/logout
    ?id_token_hint=<the id_token the module received at login>
    &client_id=<client_id>
    &post_logout_redirect_uri=<a registered logout URL>
    &state=<optional>
```

The portal ends its SSO session and redirects to `post_logout_redirect_uri?state=...`. The logout URL must be registered, like the callback. The portal keeps what it needs for this for 24 hours after the login.

Logging out of the portal itself (the "Изход" button) also ends the SSO session. Losing the portal token in one tab (it expired, or the tab is new) does not: the token lives per tab, the SSO session per browser.

Not done: single logout, where logging out of one module also ends the user's session in every other module.

## Testing a module locally

Module developers run the portal on their own machine as a single sign-on mock. It is the real portal image with the `mock` profile. It has the same login page, endpoints and tokens as production, and it comes ready to use:

- **Client id** `local-module`, **client secret** `local-secret`.
- **Any callback URL** works, registered or not, for login and logout.
- **Test users**, all with the password `password`:

| Email | Role |
|---|---|
| `student1@student.tu-sofia.bg` to `student5@student.tu-sofia.bg` | Student (`student4` is `INTERRUPTED`) |
| `ivan.petrov@tu-sofia.bg`, `maria.ivanova@tu-sofia.bg`, `georgi.dimitrov@tu-sofia.bg` | Teacher |
| `admin@tu-sofia.bg` | Admin |

You need Docker. Take [`mock/docker-compose.yml`](../mock/docker-compose.yml) from this repository and run:

```bash
docker compose -f docker-compose.yml up -d
```

The portal is then at `http://localhost:8080`. Point the module's OIDC library at:

| Setting | Value |
|---|---|
| Issuer / discovery | `http://localhost:8080` (`/.well-known/openid-configuration`) |
| Client id | `local-module` |
| Client secret | `local-secret` |
| Client authentication | `client_secret_basic` |
| Scopes | `openid profile email unicloud` |
| PKCE | `S256` (required) |

Stop it with `docker compose -f docker-compose.yml down`. Add `-v` to also delete its database and keys.

**Settings.** Set these as environment variables or in a `.env` file next to the compose file:
- `PORTAL_PORT` moves the mock off port 8080. The issuer moves with it, for example `http://localhost:9000`.
- `MOCK_CLIENT_ID`, `MOCK_CLIENT_SECRET` and `MOCK_USER_PASSWORD` change the defaults.
- `PORTAL_IMAGE` picks another portal build.

**The image.** By default the compose file runs `ghcr.io/rata5/unicloud-portal:latest`, which the pipeline publishes on every release to production. So the mock always behaves like production, apart from the callback rule. To try unreleased portal changes, build the image from this repository (`docker build -t unicloud-portal .`) and run with `PORTAL_IMAGE=unicloud-portal`.

**A module that runs in Docker too.** Tokens name `http://localhost:8080` as their issuer, because that is the address the browser uses. Inside a container, `localhost` is the container itself. So let the module's backend reach the portal at `http://host.docker.internal:8080`, but keep `http://localhost:8080` as the expected issuer. Most OIDC libraries let you set the issuer and the endpoint addresses apart.

**What is different from production.** Any callback is accepted. Every user has the same password. The session cookie is called `UNICLOUD_SSO` and has no `Secure` flag, so it works on plain `http://localhost` in every browser. Never run the `mock` profile on a real server.

## Registering a module (portal admins)

A portal admin registers each module once:

```
POST <portal>/api/v1/sso/clients
Authorization: Bearer <an admin's portal token>
Content-Type: application/json

{
  "clientId": "library",
  "name": "University library",
  "redirectUris": ["https://library.tu-sofia.bg/auth/callback"],
  "postLogoutRedirectUris": ["https://library.tu-sofia.bg/logged-out"]
}
```

Response `201`:

```json
{
  "clientId": "library",
  "clientSecret": "<43 random characters>",
  "redirectUris": ["https://library.tu-sofia.bg/auth/callback"],
  "postLogoutRedirectUris": ["https://library.tu-sofia.bg/logged-out"]
}
```

- **The secret is shown once.** The portal stores only its BCrypt hash. Send it to the module team over a private channel.
- `clientId`: 2 to 63 lower-case letters, digits or dashes. Registering the same one twice answers `409`. `unicloud-api` is reserved: it is the audience of the portal's own tokens.
- Every URL must follow the environment's rule (see [Callback URLs per environment](#callback-urls-per-environment)). In production that means `https` on a listed host; the answer is `400` otherwise. Everywhere: no fragments, user info, `*` or `,`.
- Module teams do not need a registration to develop: they use the [local mock](#testing-a-module-locally).
- Only the `ADMIN` role may register. Anyone else gets `403`. Every registration is logged with the admin's user id.
- A module may ask for the scopes `openid`, `profile`, `email` and `unicloud`.

## How the portal does it

For the portal team.

- **Library.** Spring's OAuth 2.0 Authorization Server, part of Spring Security 7 (`spring-boot-starter-security-oauth2-authorization-server`). Configuration: `sso/config/SsoAuthorizationServerConfig.java`. Its filter chain runs before the API chain and matches only the endpoints above.
- **The SSO session.** `/oauth2/authorize` is a browser redirect and cannot carry the portal's Bearer token, which lives in the page's `sessionStorage`. So the SSO endpoints use a session cookie, `__Host-UNICLOUD_SSO` (`HttpOnly; Secure; SameSite=Lax`). It ends after 30 minutes idle, and `SsoSessionMaxAgeFilter` ends it 8 hours after the password (`app.sso.session-max-age`), so a browser left logged in on a shared lab PC does not stay logged in for good. Spring Session stores it in the database, so every replica sees it. The `__Host-` prefix stops a sibling subdomain from planting the cookie.
- **Getting from a Bearer login to the session.** A browser without a session lands on `/login.html?sso=1`. After the normal login, the page calls `POST /api/v1/auth/sso-session` with its Bearer token (`sso/controller/SsoSessionController.java`). That creates the session with a new id and returns the saved `/oauth2/authorize` path, and the page opens it. The endpoint needs the Bearer header, so another site cannot trigger it with the user's cookie. It only ever returns an `/oauth2/authorize` path, so it cannot become an open redirect. If the user is already logged in to the portal, the page does this without asking for the password.
- **The API stays stateless.** The API chain never reads the session, so the cookie cannot authenticate `/api/v1/...` calls. Portal tokens have `aud=unicloud-api`; module tokens have `aud=<client_id>` and the portal's URL as `iss`. Each side refuses the other's. In the SSO chain only `/userinfo` reads a Bearer token; `/oauth2/authorize` answers only to the SSO session.
- **Callback rule.** `SsoCallbackPolicy` holds the rule (`app.sso.callback-policy`, the host list). `SsoCallbackValidators` applies it to `/oauth2/authorize` and `/connect/logout`. In `any` mode it adds the request's callback to a copy of the module's registration for the library's check only, so the library's other checks (PKCE, scopes) still run. Nothing is saved.
- **The local mock.** The `mock` profile (`application-mock.properties`) and `sso/mock/MockSsoSeeder.java`, which sets the known password and registers the known client at every start. The pipeline pushes the image as `:latest` on every release to production, next to the commit's own tag.
- **Storage.** Flyway migration `V4__sso.sql` (and its PostgreSQL copy) adds the tables: `oauth2_registered_client`, `oauth2_authorization`, `oauth2_authorization_consent`, `SPRING_SESSION` and `SPRING_SESSION_ATTRIBUTES`. `SsoAuthorizationCleanup` deletes logins older than a day, every hour.
- **Single-use codes across replicas.** `SingleUseCodeAuthorizationService` adds a conditional `UPDATE` on `code_redeemed`, so only one exchange of a code can save its tokens. The losing exchange also revokes the winner's tokens.
- **Keys.** The same RSA key pair as the portal's own tokens, published with its RFC 7638 thumbprint as `kid`.
- **Issuer.** Production sets `APP_SSO_ISSUER` to `https://<the app's address>` on every deploy. Elsewhere the portal works it out from the request, using `X-Forwarded-Proto` from the proxy, and logs a warning at startup. On staging, set `APP_SSO_ISSUER` in Render to staging's `https` address, so the issuer never depends on the proxy.
- **Time zones.** The library stores token times as `TIMESTAMP`. The MySQL connection uses `preserveInstants=true`, `connectionTimeZone=UTC` and `forceConnectionTimeZoneToSession=true` so they stay exact.
- **Tests.** `sso/SsoFlowTests.java` runs the whole flow as a module would, through the real filters and database. `sso/SsoAuthorizationStoreTests.java` covers the single-use race and the cleanup. `sso/SsoSessionMaxAgeFilterTests.java` covers the 8-hour limit. `SsoCallbackPolicyTests`, `SsoAnyCallbackTests` and `SsoMockProfileTests` cover the callback rules and the mock.

## Not done yet

- **Listing, changing and deleting modules, and rotating a secret.** Today an admin can only register. Until there is an endpoint, a secret is replaced by deleting the row in `oauth2_registered_client` and registering again.
- **Signed client authentication (`private_key_jwt`).** A module would sign a short JWT with its own private key instead of sending a shared secret. Stronger, but more work for the module teams.
- **Rate limiting.** Secrets are 256 random bits, so guessing one is not practical, but there is no limit on attempts at `/oauth2/token` (each one costs a BCrypt check), and every `GET /oauth2/authorize` without a session writes a session row. Best done at the ingress.
- **Session reads on API calls.** A browser that has an SSO session sends its cookie with every API call too. The API ignores it for authentication, but Spring Session still reads that session row, and may update it, on those calls. This is extra load on the database at 08:00. A fix is to stop the API chain from touching the session.
- **Single logout** across modules (see [Logout](#logout)).
- **Key rotation.** Tokens already carry a `kid`, so a second key can be published next to the first when the time comes.
