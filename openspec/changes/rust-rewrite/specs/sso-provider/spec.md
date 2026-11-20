## ADDED Requirements

### Requirement: Discovery document
`GET /.well-known/openid-configuration` SHALL return JSON with `issuer`, `authorization_endpoint` (`/oauth2/authorize`), `token_endpoint` (`/oauth2/token`), `jwks_uri` (`/oauth2/jwks`), `userinfo_endpoint` (`/userinfo`), `end_session_endpoint` (`/connect/logout`), `response_types_supported` `["code"]`, `grant_types_supported` `["authorization_code", "client_credentials"]`, `code_challenge_methods_supported` `["S256"]`, `subject_types_supported` `["public"]`, `id_token_signing_alg_values_supported` `["RS256"]`, `token_endpoint_auth_methods_supported` `["client_secret_basic"]` and `scopes_supported` `["openid", "profile", "email", "unicloud", "directory"]`. Endpoint URLs MUST be absolute and start with the issuer. It MUST NOT advertise endpoints the portal does not serve.

#### Scenario: Library reads discovery
- **WHEN** an OIDC client library fetches the discovery document
- **THEN** every listed endpoint answers, and `issuer` equals the configured issuer exactly

### Requirement: Issuer
The issuer SHALL be `APP_SSO_ISSUER` when set, stripped and used as is. When it is not set, the portal SHALL derive it from the request scheme and host, honouring `X-Forwarded-Proto` and `X-Forwarded-Host`, and log a warning at startup.

#### Scenario: Configured issuer
- **WHEN** `APP_SSO_ISSUER` is `https://portal.example`
- **THEN** discovery `issuer` and every token's `iss` are `https://portal.example`

### Requirement: JWKS and signing key
The portal SHALL sign tokens with RS256 using one RSA key pair given as PEM text in environment variables. `GET /oauth2/jwks` SHALL publish the public key only (`kty`, `n`, `e`, `kid`, never `d`). `kid` MUST be the RFC 7638 SHA-256 thumbprint of the public key, base64url without padding. Every token header SHALL carry `alg: RS256` and that `kid`.

#### Scenario: Thumbprint
- **WHEN** the key is the RSA example key from RFC 7638 section 3.1
- **THEN** `kid` is `NzbLsXh8uDCcd-6MNwXF4W_7noWXFZAfHkxZsRGC9Xs`

#### Scenario: No private part
- **WHEN** a client fetches `/oauth2/jwks`
- **THEN** no key has a `d`, `p`, `q`, `dp`, `dq` or `qi` member

### Requirement: Authorize without a session
`GET /oauth2/authorize` from a browser without an SSO session SHALL save the request in a new session and answer 302 to the sign-in page. After sign-in (and a forced password change, if due) the portal SHALL continue the saved request. A Bearer header MUST NOT authenticate this endpoint.

#### Scenario: First visit
- **WHEN** a browser with no session opens a valid authorize URL
- **THEN** the portal answers 302 to the sign-in page and sets `__Host-UNICLOUD_SSO`
- **AND** after a correct sign-in the browser lands on the module's callback with a code

#### Scenario: Bearer header ignored
- **WHEN** a request to `/oauth2/authorize` carries a valid module access token but no session cookie
- **THEN** the portal still redirects to the sign-in page

### Requirement: Authorize with a session
With a live SSO session and a valid request, `/oauth2/authorize` SHALL answer 302 to `<redirect_uri>?code=<code>&state=<state>`, with `state` echoed verbatim when sent and omitted when not. There is no consent screen.

#### Scenario: Silent login
- **WHEN** a signed-in user opens a valid authorize URL for any registered module
- **THEN** the portal redirects straight to the callback with a code, without asking for a password

### Requirement: Plain 400 for untrusted callbacks
The portal MUST answer a plain 400 page, and MUST NOT redirect, when `client_id` is unknown, when `redirect_uri` is missing and the module does not have exactly one registered callback, when `redirect_uri` is not allowed by the callback policy, or when it is not an absolute `http` or `https` URL.

#### Scenario: Unregistered callback in production
- **WHEN** a production authorize request uses a callback that is not registered for the module
- **THEN** the portal answers 400 with no `Location` header

#### Scenario: Path suffix
- **WHEN** the registered callback is `https://library.tu-sofia.bg/auth/callback` and the request uses `https://library.tu-sofia.bg/auth/callback/../admin`
- **THEN** the portal answers 400 with no redirect

#### Scenario: Javascript URL on staging
- **WHEN** a staging authorize request uses `javascript:alert(1)` as callback
- **THEN** the portal answers 400 with no redirect

### Requirement: Errors returned to a trusted callback
Once the callback is trusted, request errors SHALL be returned as 302 to `<redirect_uri>?error=<code>&error_description=<text>&state=<state>`: `invalid_request` when `code_challenge` is missing, `code_challenge_method` is not `S256`, or `openid` is missing from `scope`; `invalid_scope` for a scope outside `openid profile email unicloud`; `unsupported_response_type` when `response_type` is not `code`.

#### Scenario: Missing PKCE
- **WHEN** a signed-in user opens an authorize URL without `code_challenge`
- **THEN** the redirect starts with `<redirect_uri>?error=invalid_request`

#### Scenario: Plain PKCE
- **WHEN** the request uses `code_challenge_method=plain`
- **THEN** the redirect carries `error=invalid_request`

#### Scenario: Unknown scope
- **WHEN** the request asks for scope `openid admin`
- **THEN** the redirect carries `error=invalid_scope`

### Requirement: Authorization codes
A code SHALL be opaque, random (at least 128 bits), bound to the module, the callback, the PKCE challenge, the nonce and the granted scopes, valid for 60 seconds, and usable once.

#### Scenario: Expired code
- **WHEN** a module exchanges a code 61 seconds after it was issued
- **THEN** the token endpoint answers 400 `invalid_grant`

### Requirement: Token endpoint for authorization codes
`POST /oauth2/token` SHALL accept `application/x-www-form-urlencoded` with `grant_type=authorization_code`, `code`, `redirect_uri` and `code_verifier`, and client authentication by HTTP Basic only. On success it SHALL answer 200 JSON `{"access_token", "id_token", "scope", "token_type": "Bearer", "expires_in"}` with `Cache-Control: no-store`. There MUST be no refresh token. Errors SHALL be JSON `{"error", "error_description"}`: 401 `invalid_client` for an unknown client, a wrong secret or credentials not sent by Basic; 400 `invalid_grant` for a bad, expired or used code, a `redirect_uri` that differs from the authorize request, a verifier that does not hash to the challenge, or a user who is now disabled or deleted; 400 `unsupported_grant_type` for any other grant type.

#### Scenario: Successful exchange
- **WHEN** the module sends a fresh code, the same callback, the right verifier and its secret by Basic
- **THEN** the answer is 200 with an `id_token`, an `access_token` and `expires_in` close to 1800

#### Scenario: Secret in the body
- **WHEN** the module sends `client_id` and `client_secret` as form fields and no Basic header
- **THEN** the answer is 401 `invalid_client`

#### Scenario: Wrong verifier
- **WHEN** the verifier does not match the challenge
- **THEN** the answer is 400 `invalid_grant`

#### Scenario: Disabled user
- **WHEN** the user is disabled between the code and the exchange
- **THEN** the answer is 400 `invalid_grant` and contains no `id_token`

### Requirement: Code replay revokes tokens
A second exchange of the same code MUST fail with 400 `invalid_grant` and MUST revoke the access token issued by the first exchange. This MUST hold when two replicas receive the same code at the same moment: exactly one exchange succeeds, and its access token is then revoked.

#### Scenario: Replay
- **WHEN** a code is exchanged twice
- **THEN** the second exchange gets `invalid_grant`
- **AND** `/userinfo` with the first access token answers 401

#### Scenario: Concurrent replay
- **WHEN** two exchanges of one code arrive at two replicas at the same moment
- **THEN** at most one gets 200, the other gets `invalid_grant`, and any issued access token is revoked

### Requirement: ID token
The ID token SHALL be a JWT with `iss`, `sub` (the user id as a JSON string), `aud` (an array with the module's client id), `azp` (the client id), `iat`, `exp` (30 minutes after `iat`), `auth_time` (integer seconds when the password was typed), `nonce` when one was sent, and the user claims of the granted scopes, read from the database when the token is issued.

#### Scenario: Claim types
- **WHEN** user 4 logs in with nonce `n-1`
- **THEN** the ID token has `"sub": "4"`, `aud` as an array containing the client id, and `"nonce": "n-1"`

### Requirement: User claims by scope
Scope `email` SHALL give `email`. Scope `profile` SHALL give `given_name`, `family_name` and `name` (given and family name joined by one space). Scope `unicloud` SHALL give `role`, `permissions`, `faculty` (`{"code", "name"}`, only when the user has a faculty) and `student` (`{"faculty_number", "specialty", "study_year", "status"}`, only for students; `study_year` is an integer). Claims of scopes not granted MUST be absent.

#### Scenario: Narrow scopes
- **WHEN** a module asks for `openid email`
- **THEN** the ID token has `email` and has no `family_name`, `role`, `faculty` or `student`

#### Scenario: Admin has no faculty
- **WHEN** an admin logs in with scope `unicloud`
- **THEN** the ID token has `role` and `permissions` and no `faculty`

### Requirement: Access token
The access token SHALL be an RS256 JWT with `iss`, `sub`, `aud` (an array with the client id), `client_id`, `scope`, `iat`, `exp` (30 minutes) and `jti`. It SHALL work only at `/userinfo` (authorization-code tokens) or at the directory API (client-credentials tokens with scope `directory`).

#### Scenario: Token on a portal page
- **WHEN** a module sends its access token as Bearer to `/profile`
- **THEN** the portal treats the request as anonymous

### Requirement: Userinfo
`GET /userinfo` and `POST /userinfo` SHALL accept a Bearer access token from an authorization-code exchange and return JSON with `sub` and the same user claims as the ID token for the granted scopes, read from the database at request time. They MUST answer 401 for a missing, invalid, expired or revoked token, for a client-credentials token, and for a user who is now disabled or deleted.

#### Scenario: Live claims
- **WHEN** a student's study year changes after login and the module calls `/userinfo`
- **THEN** `student.study_year` shows the new value

#### Scenario: ID token sent instead
- **WHEN** a module sends its ID token as Bearer to `/userinfo`
- **THEN** the answer is 401

### Requirement: Logout everywhere
`GET /connect/logout` with `id_token_hint`, `client_id`, `post_logout_redirect_uri` and optional `state` SHALL end the browser's SSO session and answer 302 to exactly `post_logout_redirect_uri?state=<state>`, or to exactly `post_logout_redirect_uri` when no state is sent. The logout URL MUST follow the callback policy. The portal MUST be able to resolve an `id_token_hint` for at least 24 hours after the login, even after the token expired.

#### Scenario: Logout with state
- **WHEN** a module sends the user to logout with a registered logout URL and `state=xyz`
- **THEN** the portal redirects to `<logout URL>?state=xyz`
- **AND** the next authorize from that browser asks for the password

#### Scenario: Unregistered logout URL in production
- **WHEN** the logout URL is not registered for the module in production
- **THEN** the portal answers 400 and does not redirect

#### Scenario: Hint after 23 hours
- **WHEN** a module sends an `id_token_hint` from a login 23 hours ago
- **THEN** logout succeeds

### Requirement: Callback policy
The portal SHALL run in one of two callback modes, set by configuration. `allowlist` (production default): a callback or logout URL MUST be registered for the module, match character for character, use `https` (or `http` on a listed `localhost`, `127.0.0.1` or `[::1]`), and have a host on the allowed host list. `any` (staging): any absolute `http` or `https` URL is accepted, registered or not, and nothing is saved. An unknown mode MUST stop startup. The host check MUST run at every authorize and logout, not only at registration.

#### Scenario: Host removed
- **WHEN** a registered module's host is removed from the list and the portal restarts
- **THEN** its authorize requests get a plain 400

#### Scenario: Staging localhost
- **WHEN** a staging authorize request uses `http://localhost:3000/auth/callback`, which is not registered
- **THEN** the login completes and the browser lands on that URL with a code

### Requirement: Allowed host list
The host list SHALL come from a file in the repository, comma-separated, and the environment variable `APP_SSO_ALLOWED_CALLBACK_HOSTS` SHALL replace it entirely when set. Entries are stripped and lower-cased, blanks dropped. An entry is an exact host, or `*.domain`, which matches any host ending in `.domain` at any depth but not `domain` itself. Matching MUST ignore letter case, port and path.

#### Scenario: Look-alike hosts
- **WHEN** the list is `*.tu-sofia.bg`
- **THEN** `a.b.tu-sofia.bg` is allowed
- **AND** `tu-sofia.bg`, `evil-tu-sofia.bg` and `library.tu-sofia.bg.evil.example` are refused

### Requirement: Module registration
An admin SHALL register a module on a portal page (cookie session and CSRF token) with a client id, a name, one or more callback URLs, optional logout URLs, and an optional `directory` grant. The portal SHALL show the client secret once, right after registration, and store only its hash. Rules: `clientId` matches `[a-z0-9][a-z0-9-]{1,62}`; `unicloud-api` is reserved; a taken id is refused; `name` is 1 to 200 characters; each URL is stripped and de-duplicated, follows the callback policy, and has no fragment, user info, `*` or `,`; each URL list joined by commas is at most 1000 characters. The secret SHALL be 32 random bytes, URL-safe base64 without padding (43 characters). The registration MUST be logged with the admin's user id.

#### Scenario: Register
- **WHEN** an admin registers `library` with callback `https://library.tu-sofia.bg/auth/callback` on an allowed host
- **THEN** the page shows a 43-character secret once
- **AND** reloading the page does not show it again

#### Scenario: Reserved id
- **WHEN** an admin registers `unicloud-api`
- **THEN** the form refuses it

#### Scenario: Leading dash
- **WHEN** an admin registers `-library`
- **THEN** the form refuses it

#### Scenario: Not an admin
- **WHEN** a user without role `ADMIN` posts the registration form
- **THEN** the portal answers 403

### Requirement: Login data retention
The portal SHALL delete stored login data (codes, issued token records, logout hints) older than 24 hours, checking at least every hour.

#### Scenario: Old rows
- **WHEN** the cleanup runs
- **THEN** login records created more than 25 hours ago no longer exist
