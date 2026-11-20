# SSO wire contract

This file records exactly what the Java portal sends and accepts for single sign-on. It is the reference for the Rust conformance suite. The readable guide for module teams is [sso.md](sso.md).

**Source.** The Java reference implementation at commit `93365ec`. The Java code is removed by the Rust rewrite, so every `file:line` below points at that commit. Paths are relative to `src/main/java/bg/tusofia/unicloud/portal/` (main code) or `src/test/java/bg/tusofia/unicloud/portal/` (tests).

**Unverified (Spring default).** The Java portal configures Spring Authorization Server (Boot 4.1.2, `pom.xml:8`, starter at `:95`). Items marked this way come from the library's defaults, not from repo code or tests. They must be confirmed against a recording of the Java staging before the cutover: discovery, JWKS, a token response, decoded ID and access tokens, and error responses.

## 1. Discovery document

- The repo does not customize it. The only setting is `AuthorizationServerSettings` with an optional issuer (`sso/config/SsoAuthorizationServerConfig.java:176-189`). Endpoint paths are the library defaults, listed in `docs/sso.md:77-84`.
- A test pins only these: `issuer`, `authorization_endpoint`, `token_endpoint`, `userinfo_endpoint` and `end_session_endpoint` exist, and `/oauth2/jwks` returns `keys[0].kid` with no `d` (`sso/SsoFlowTests.java:438`).
- **Issuer.** `app.sso.issuer` (env `APP_SSO_ISSUER`), stripped, when not blank (`config:183-187`, `application.properties:31`). Otherwise it is worked out from the request host and `X-Forwarded-*` (`server.forward-headers-strategy=native`, `application.properties:51`). No trailing-slash normalisation.
- **Unverified (Spring default).** Expected values:

| Field | Expected value |
|---|---|
| `authorization_endpoint` | `/oauth2/authorize` |
| `token_endpoint` | `/oauth2/token` |
| `jwks_uri` | `/oauth2/jwks` |
| `userinfo_endpoint` | `/userinfo` |
| `end_session_endpoint` | `/connect/logout` |
| `response_types_supported` | `["code"]` |
| `code_challenge_methods_supported` | `["S256"]` |
| `subject_types_supported` | `["public"]` |
| `id_token_signing_alg_values_supported` | `["RS256"]` |
| `scopes_supported` | `["openid"]` |

- **Unverified (Spring default).** The default document also advertises extra grant types (`client_credentials`, `refresh_token`, device code, token exchange), extra client authentication methods (post, `client_secret_jwt`, `private_key_jwt`, tls) and revocation, introspection and device endpoints. Clients are registered with only `authorization_code` and `client_secret_basic` (`sso/application/SsoClientRegistrationService.java:89-90`), so the extras are advertised but unusable.

## 2. `/oauth2/authorize`

- **Without an SSO session** (`config:112-117`):
  - A request with `Accept: text/html` gets a 302 to `/login.html?sso=1` (`LOGIN_PAGE`, `config:67`). The Location is absolute. Tests assert `endsWith("/login.html?sso=1")` (`SsoFlowTests:110`).
  - The request is saved by the default `HttpSessionRequestCache`. The response sets the `__Host-UNICLOUD_SSO` cookie.
  - Unverified (Spring default): a non-HTML `Accept` has no matching entry point, probably 403. Every test sends `Accept: text/html`.
  - A Bearer token never authenticates here, neither a portal token nor a module token. It still gets the login redirect (`config:131-138`, `SsoFlowTests:339`).
- **Resume.** The login page calls `POST /api/v1/auth/sso-session`, which returns `continueUrl` (section 6). The page then opens `/oauth2/authorize?<original raw query>` again.
- **Plain 400, no redirect to the callback:**
  - The callback fails the library's registration check: not registered, or not an exact match. Unverified (Spring default) as code, but pinned by `SsoFlowTests:250` (unregistered), `:261` (`.../../admin` suffix) and `:270` (host removed from the list).
  - The callback host or scheme fails `SsoCallbackPolicy.permits`, in `allowlist` mode only (`sso/infrastructure/SsoCallbackValidators.java:52-57`). It throws `invalid_request` with a null authentication result, which means no redirect. In `any` mode, an unregistered callback that is not an http(s) URL falls through to the library check and also gets a plain 400.
  - Unknown `client_id`. Unverified (Spring default); `docs/sso.md:317` agrees.
  - Missing `redirect_uri` when the scope has `openid` or more than one URI is registered. Unverified (Spring default). With exactly one registered URI and none sent, the host check runs on that one (`SsoCallbackValidators:53`).
- **Redirect to the callback with `?error=...&state=...`:**
  - Missing PKCE gives `error=invalid_request`. The Location starts with `<redirect_uri>?error=invalid_request` (`SsoFlowTests:302`).
  - Other errors come from the library validator (`SsoCallbackValidators:23-25,38`). `invalid_scope` for a scope the client is not registered for (`docs/sso.md:319`) and `unsupported_response_type` are Unverified (Spring default). No test pins them.
- **PKCE.** `requireProofKey(true)` on every client (`SsoClientRegistrationService:97-100`). Unverified (Spring default): only S256 is accepted, and `plain` gets `invalid_request`.
- **Scopes.** Registered scopes are `openid`, `profile`, `email` and `unicloud` (`SsoClientRegistrationService:93-96`). The library checks the request against that set (Unverified, Spring default). The repo does not enforce that `openid` is present. Consent is off (`:99`), so there is no consent screen.
- **state and nonce.**
  - `state` is optional (Unverified, Spring default) and echoed verbatim. A success is `?code=...&state=...` (`SsoFlowTests:110`).
  - `nonce` is optional. `SsoAnyCallbackTests:90-98` omits it and succeeds. When sent, it is in the ID token (`SsoFlowTests:110`).
  - Both travel in the original query that is saved and replayed through `continueUrl`.
- **Code.** Lives 60 seconds (`application.properties:33`). Single use.

## 3. `/oauth2/token`

- **Request.** `POST`, `application/x-www-form-urlencoded`, `Authorization: Basic base64(client_id:client_secret)` (`SsoFlowTests` helper `exchange`, about line 520). Fields: `grant_type=authorization_code`, `code`, `redirect_uri` (must equal the one used at authorize) and `code_verifier`.
- **Client authentication.** Only `CLIENT_SECRET_BASIC` is registered (`SsoClientRegistrationService:89`). `client_secret_post` is therefore rejected with `invalid_client` (inferred, no test). The only grant is `authorization_code` (`:90`): no refresh token, no `client_credentials`. No refresh TTL is set.
- **Response.** No repo code or test pins the body. Unverified (Spring default), plus the example in `docs/sso.md:149-157`:
  - `access_token` (JWT), `id_token` (JWT), `scope` (space-separated string), `token_type` `"Bearer"`, `expires_in` (number; the doc shows 1799).
  - No `refresh_token`.
- **Lifetimes.**
  - Access token: 30 minutes, from `app.jwt.ttl-minutes=30` (`application.properties:22`, `SsoClientRegistrationService:53,60,103`).
  - ID token: 30 minutes. Unverified (Spring default): a library constant, not tied to the setting.
  - Code: 60 seconds (`:102`).
- **Errors.** JSON `{"error": "..."}`. Pinned:
  - 400 `invalid_grant` for a wrong verifier (`SsoFlowTests:227`), a replayed code (`:110`) and a disabled user (`:365`).
  - 401 `invalid_client` for a wrong secret (`:238`).
  - `error_description` and any `error_uri` are Unverified (Spring default). The disabled-user description is "The user no longer exists or is disabled" (`config:214-217`).
- **Disabled or deleted user.** The ID-token customizer throws, so the response is 400 with no `id_token` (`config:193-203`, `SsoFlowTests:365`).
- **Double redemption** (`sso/infrastructure/SingleUseCodeAuthorizationService.java`):
  - Replay after the first exchange: 400 `invalid_grant`. The first exchange's access token then gets 401 at `/userinfo` (`SsoFlowTests:110`, step 6). The revocation itself is the library's replay handling (Unverified, Spring default).
  - Race between replicas (`:38-49`): the winner's `UPDATE oauth2_authorization SET code_redeemed=TRUE WHERE id=? AND code_redeemed=FALSE` (`:28-29`) takes effect. The loser gets `invalid_grant` ("The authorization code has already been used"). The loser's save also invalidates the winner's stored access token and ID token (`:55-69`).
  - Revocation only marks the stored token as invalid, and only `/userinfo` checks that store. The JWTs themselves still verify by signature until `exp`.
  - Later saves of the same authorization, such as logout, pass through (`:71-91`, `SsoAuthorizationStoreTests:87`).

## 4. Tokens

- **Signing.** RS256 with the same RSA key pair as the portal API, from `app.jwt.private-key` and `app.jwt.public-key` (`auth/infrastructure/security/config/SecurityConfig.java:37-41,127-135`).
- **kid.** `RSAKey.Builder(...).keyIDFromThumbprint()` (`SecurityConfig:130-132`): the RFC 7638 SHA-256 thumbprint, base64url. One key. The JWKS shows only the public part, with no `d` (`SsoFlowTests:438`). Whether `use` and `alg` are absent is not confirmed; the builder does not set them.
- **ID token claims:**
  - `sub` is a string: `String.valueOf(userId)` (`sso/controller/SsoSessionController.java:87-88`). Tests assert `"4"` (`SsoFlowTests:110`).
  - `aud` is an array that contains the client id (`SsoFlowTests:110`, `docs/sso.md:188`).
  - `iss` is the issuer from section 1.
  - `nonce` is present when one was sent.
  - `auth_time` is not pinned (the doc example is the integer `1795031651`). It is derived from the portal JWT's `iat` through the password-factor authority (`SsoSessionController:87-94`). Whether the library emits it is Unverified (Spring default).
  - `exp`, `iat`, `sid`, `azp` and the header fields are Unverified (Spring default).
  - User claims come from `SsoUserClaims` (`sso/application/SsoUserClaims.java:57-74`) and go into the ID token only (`config:193-203`):

| Scope | Claim | JSON type |
|---|---|---|
| `email` | `email` | string |
| `profile` | `given_name`, `family_name` | string |
| `profile` | `name` | string, `first + " " + last` |
| `unicloud` | `role` | string, the enum name |
| `unicloud` | `permissions` | array of strings, in bit order |
| `unicloud` | `student` | object, only when a student row exists |

  - `student` fields: `faculty_number` (string), `specialty` (string), `study_year` (integer), `status` (string: `ACTIVE`, `INTERRUPTED` or `GRADUATED`) (`SsoUserClaims:84-91`, `docs/sso.md:211`).
  - Claims are read from the database when the token is issued. `SsoFlowTests:314` pins that with scopes `openid email`, the claims `family_name`, `role` and `student` are absent.
- **Access token.** A JWT. Its claims are Unverified (Spring default): expected `iss`, `sub` (the same user id string), `aud` `[client_id]`, `exp`, `iat`, `nbf`, `jti` and `scope`. It carries no user claims. It works only at `/userinfo`. `/api/v1/**` refuses it with 401 (`SsoFlowTests:110`), because the API wants `iss=unicloud-portal` and an `aud` that contains `unicloud-api`. That is why the client id `unicloud-api` is reserved (`SsoClientRegistrationService:71-75`).
- **Header.** `alg=RS256` and `kid` are certain. Any `typ` is Unverified (Spring default).

## 5. `/userinfo` and `/connect/logout`

- **`/userinfo`:**
  - A Bearer token is honoured only when the path is exactly `/userinfo`, from the header (`config:134-137`). The decoder is the library's `jwtDecoder(jwkSource)` (`:122`), and the library then looks the token up in its store.
  - A portal API token gets 401 (`SsoFlowTests:454`). A revoked token gets 401 (`:110`).
  - The answer is JSON: `sub` plus the same claims as the ID token, filtered by the granted scopes and read live from the database (`config:205-212`). Pinned: `$.sub` is `"4"`, `family_name` is `"Стоянов"`, `student.study_year` is `2`.
  - A disabled or deleted user raises `invalid_grant`. Its HTTP status is Unverified (Spring default).
  - Unverified (Spring default): `POST /userinfo` with a Bearer header probably also works.
- **`/connect/logout`:**
  - Inputs: `id_token_hint`, `client_id`, `post_logout_redirect_uri` and `state` (`SsoFlowTests:193`), plus the SSO cookie. The `id_token_hint` is looked up in `oauth2_authorization`.
  - Success is a 302 to exactly `post_logout_redirect_uri + "?state=" + state` (`SsoFlowTests:193`). With no state, the redirect is exactly the target, with no `?` (`SsoAnyCallbackTests:128-135`). The SSO session ends, and the next authorize goes to the login page.
  - In `allowlist` mode the URL must match a registered `postLogoutRedirectUris` entry exactly, and `policy.permits` must also hold. Otherwise `invalid_request` with "OAuth 2.0 Parameter: post_logout_redirect_uri" (`SsoCallbackValidators:61-81`). In `any` mode any absolute http(s) URL works, added to a copy of the client (`:68-74`). Without `post_logout_redirect_uri`, the library's behaviour applies (Unverified, Spring default).
  - The HTTP shape of logout errors is Unverified (Spring default).
  - **The 24-hour rule.** `SsoAuthorizationCleanup.RETENTION` is 1 day (`sso/infrastructure/SsoAuthorizationCleanup.java:24`). The job runs every hour after a 5-minute delay (`:36`): `DELETE ... WHERE created_at < now - 1 day` (`:28,38`). `created_at` defaults to the insert time (`V4__sso.sql:67`). So logout data lives 24 to about 25 hours. After that, `id_token_hint` cannot be resolved. The library never deletes these rows itself (`:14-19`).

## 6. Session cookie and `/api/v1/auth/sso-session`

- **Cookie** (`config:74,144-153`): name `__Host-UNICLOUD_SSO`, `Path=/`, no `Domain`, `HttpOnly`, `Secure`, `SameSite=Lax`. The value is a Spring Session id stored in `SPRING_SESSION` and `SPRING_SESSION_ATTRIBUTES` (`V4__sso.sql:85-107`), base64-encoded (Unverified, Spring default). A new session id is issued at sign-in (`SsoSessionController:78-81`, `SsoFlowTests:110`).
- **Lifetimes.**
  - Idle timeout: 30 minutes (`application.properties:42`). Every request resets it (`docs/sso.md:385`).
  - Maximum age: 8 hours (`application.properties:44`), counted from the portal JWT's `iat`, stored as the `LOGIN_AT` session attribute (`SsoSessionController:97`).
  - `SsoSessionMaxAgeFilter` (`sso/infrastructure/SsoSessionMaxAgeFilter.java:38-52`) ends a session when `loginAt + maxAge` is strictly before now. It invalidates the session and the request goes on as anonymous. A session without the attribute never expires by this rule. The filter runs after `SecurityContextHolderFilter` (`config:123-125`).
- **`POST /api/v1/auth/sso-session`** (`SsoSessionController:61-100`):
  - Runs in the API chain. Needs `Authorization: Bearer <portal JWT>`. No body.
  - 401 for a missing or invalid token (`SsoFlowTests:389`). The body is the resource-server default, probably empty (Unverified, Spring default).
  - 401 with an empty body when `sub` is not a numeric user id, or the user is missing or disabled (`:67-73`, `SsoFlowTests:382`).
  - Otherwise 200 with `{"continueUrl": <string or null>}` (`controller/dto/SsoSessionResponse.java:8`). `null` is sent as is. The test at `:395` asserts `doesNotExist`, which also passes for null.
  - `continueUrl` is the saved request as `/oauth2/authorize?<raw query>`. It is null if nothing is saved or the saved path is not exactly `/oauth2/authorize` (`:112-125`). The saved request is then removed (`:76`).
  - Principal name: `String.valueOf(user.getId())`. Authorities: `ROLE_<role>` plus a password-factor authority carrying the JWT `iat` (`:87-94`).
  - The response sets the new session cookie.
- **`DELETE /api/v1/auth/sso-session`** is open to all (`SecurityConfig:90-93`). Always 204. It ends the session if there is one (`SsoSessionController:103-110`).
- `POST /api/v1/sso/clients` sets no cookie (`SsoFlowTests`, about line 480).

## 7. Client registration

- **`POST /api/v1/sso/clients`** (`sso/controller/SsoClientController.java:30-38`):
  - Needs a Bearer portal JWT with role `ADMIN`. Anyone else gets 403 (`SsoFlowTests:403`) with the `ApiError` body (`exception/GlobalExceptionHandler.java:84-94`).
  - Success: **201** with `{"clientId", "clientSecret", "redirectUris": [...], "postLogoutRedirectUris": [...]}` (`controller/dto/RegisterSsoClientResponse.java:6-11`). The URL lists are stripped and de-duplicated.
- **Staging `POST /api/v1/staging/sso-clients`**: same body and answer, no authentication. `registeredBy` is `"staging-tools"` (`staging/StagingToolsController.java:114-118`).
- **Request validation** (`sso/controller/dto/RegisterSsoClientRequest.java:18-32`):

| Field | Rule |
|---|---|
| `clientId` | not blank, matches `[a-z0-9][a-z0-9-]{1,62}`: 2 to 63 characters, and the first is not a dash |
| `name` | not blank, at most 200 characters, stripped (`SsoClientRegistrationService:87`) |
| `redirectUris` | not empty, every element not blank |
| `postLogoutRedirectUris` | optional; null means `[]` (`:81-83`) |

- **Service checks, in this order** (`SsoClientRegistrationService:68-111`):
  1. `clientId` equal to `unicloud-api`, ignoring case: 400 "clientId: 'x' is reserved" (`:73-75`).
  2. Already registered: 409 "SSO client already exists with clientId: 'x'" (`:76-78`, `exception/DuplicateResourceException.java:9-11`).
  3. Each URL is stripped and de-duplicated, then checked (`:113-121`).
  4. The comma-joined list must be at most 1000 characters (`:35,116-119`), else 400 "<field> must not be longer than 1000 characters in total".
- **URL rules** (`sso/application/SsoCallbackPolicy.java:58-68,83-94`):
  - It parses as a URL, is absolute, has a host, and its scheme is `http` or `https`. Otherwise "not a valid URL".
  - In `allowlist` mode the scheme is `https`, or the host is `localhost`, `127.0.0.1` or `[::1]`. The host must also be on the list.
  - Then: no fragment, no user info, no `*`, no `,` (`:63-67`).
  - Messages read `"<field>: <rule>: <value>"`. The allowlist message is "must be an https URL on a host listed in sso-callback-hosts.properties (http only for a listed localhost), allowed hosts: [...]" (`:106-111`).
- **Error body** (`exception/GlobalExceptionHandler.java:63-77,96-106`, `dto/ApiError.java`): `{timestamp, status, error, message, path, fieldErrors}`. Validation failures give 400 with message "Validation failed" and `fieldErrors` entries `{field, message}`. Rule failures give 400 with the rule's message.
- **Secret** (`SsoClientRegistrationService:36,123-127`): 32 bytes from a secure random source, URL-safe base64 without padding, so 43 characters. Stored as a bare BCrypt hash `$2a$10$...` (`SecurityConfig:118-121`), with no `{bcrypt}` prefix. It is shown once, in the registration answer.
- **Stored client.** A row in `oauth2_registered_client` (`V4__sso.sql:8-24`) with a generated UUID `id` (`:85`). `client_settings` and `token_settings` hold the library's Java-typed JSON. The code and access-token lifetimes are written per client (`:101-104`).

## 8. Callback policy

- `app.sso.callback-policy` is `allowlist` by default (`application.properties:36`) and `any` under the `staging` profile (`application-staging.properties:5`). The value is stripped and upper-cased. An unknown value stops startup (`SsoCallbackPolicy.java:39`, `SsoCallbackPolicyTests:77`).
- **Host list.** `app.sso.allowed-callback-hosts`, a comma-separated list. Each entry is stripped and lower-cased, and blank entries are dropped (`SsoCallbackPolicy:37-43`). It lives in `src/main/resources/sso-callback-hosts.properties`, which is empty today (`:14`). `APP_SSO_ALLOWED_CALLBACK_HOSTS` replaces the whole list (`:11`).
- **Matching** (`SsoCallbackPolicy:96-104`):
  - An exact host, or `*.example.org`, which matches any host that ends with `.example.org`, at any depth (`a.b.tu-sofia.bg`), but not `example.org` itself.
  - Case-insensitive. Port and path are ignored by this check. The whole-URL exact match comes from the registered URL comparison.
  - `evil-tu-sofia.bg` and `library.tu-sofia.bg.evil.example` are refused (`sso/SsoCallbackPolicyTests.java:19-55`).
  - Unverified (Spring default): an RFC 8252 loopback exception may let the port differ for `127.0.0.1` and `[::1]`.
- **At runtime.** The host check also runs at authorize and logout, so taking a host off the list switches off modules that are already registered (`SsoFlowTests:270`).
- **`any` mode.** Any absolute http or https URL works for authorize and logout, registered or not (`SsoAnyCallbackTests:83-136`). It is added to a copy of the client for the check only; nothing is saved (`SsoCallbackValidators:23-26,83-99`). `javascript:`, `ftp:` and relative URLs are refused everywhere (`SsoCallbackPolicyTests:68-75`, `SsoAnyCallbackTests:139`).

## 9. The portal's own JWT

- **Sign-in.** `POST /api/v1/auth/login` with `{email, password}` (`auth/controller/dto/LoginRequest.java`). The email is stripped and lower-cased (`auth/application/AuthenticationService.java:32-33`). Answer: `{accessToken, tokenType: "Bearer", expiresIn}`, with `expiresIn` 1800 seconds (`auth/controller/AuthController.java:25-39`). A wrong password or a disabled user gives 401 `{"error": "Invalid email or password"}` (`auth/controller/AuthenticationExceptionHandler.java:14-21`).
- **Claims** (`auth/application/JwtAccessTokenIssuer.java:38-69`): `iss` `"unicloud-portal"` (a plain string), `sub` (the user id as a string), `aud` `["unicloud-api"]`, `iat`, `exp` (30 minutes later), `role` (enum name) and `permissions` (array of names in bit order, not a bitmask). No `nbf` or `jti`. Header RS256 plus `kid`.
- **Verification** (`SecurityConfig:142-161`): RS256, default time checks, `iss == "unicloud-portal"`, `aud` contains `"unicloud-api"`. Authorities: `ROLE_<role>` plus each permission name (`:163-188`).
- **Storage.** `users.permissions_mask`, `BIGINT UNSIGNED NOT NULL DEFAULT 0` (`db/migration/V3__users_permissions_mask.sql:5-6`).
- **Permissions** (`user/domain/Permission.java:10-22`), mask `1 << bit`:

| Bit | Name |
|---|---|
| 0 | `STAFF_CREATE` |
| 1 | `STUDENT_CREATE` |
| 2 | `STUDENT_PROFILE_READ_OWN` |
| 3 | `STUDENT_PROFILE_READ_ANY` |
| 4 | `STUDENT_PROFILE_READ_ENROLLED` |
| 5 | `COURSE_CREATE_MODIFY` |
| 6 | `COURSE_OFFERING_READ` |
| 7 | `COURSE_OFFERING_CREATE_MODIFY` |
| 8 | `ENROLLMENT_REQUEST_CREATE` |
| 9 | `ENROLLMENT_REQUEST_ACCEPT_REJECT` |

- **Roles** (`user/domain/Role.java:5-27`), in the order `ADMIN, INSPECTOR, TEACHER, STUDENT`. The defaults agree with the V3 backfill (`V3__users_permissions_mask.sql:10-15`):

| Role | Default permissions | Mask |
|---|---|---|
| `ADMIN` | `STAFF_CREATE` | 1 |
| `INSPECTOR` | `STUDENT_CREATE`, `STUDENT_PROFILE_READ_ANY`, `COURSE_CREATE_MODIFY`, `COURSE_OFFERING_READ`, `COURSE_OFFERING_CREATE_MODIFY` | 234 |
| `TEACHER` | `COURSE_OFFERING_READ`, `STUDENT_PROFILE_READ_ENROLLED`, `ENROLLMENT_REQUEST_ACCEPT_REJECT` | 592 |
| `STUDENT` | `STUDENT_PROFILE_READ_OWN`, `COURSE_OFFERING_READ`, `ENROLLMENT_REQUEST_CREATE` | 324 |

## 10. Staging API

The routes exist only with the `staging` profile (`staging/StagingToolsController.java:50`). The gate is at `SecurityConfig:94-98`. Without the profile, anonymous callers get 401 and admins get 403 (`staging/StagingToolsAbsentTests.java:54-74`). With it, no sign-in is needed.

| Route | Behaviour |
|---|---|
| `GET /staging` | HTML page, `Cache-Control: no-store` (`:69-75`) |
| `GET /api/v1/staging/roles` | `{"roleDefaults": {ADMIN: [...], INSPECTOR: [...], TEACHER: [...], STUDENT: [...]}, "permissions": [10 names]}` (`:77-90`). `roleDefaults` is in role order, lists in bit order. Top-level key order is not guaranteed |
| `GET /api/v1/staging/users` | Array of the user shape, ordered by id (`staging/StagingUserService.java:35-39,91-100`) |
| `POST /api/v1/staging/users` | 201 with the user shape |
| `PUT /api/v1/staging/users/{id}/permissions` | 200 with the user shape. Body `{"permissions": [...]}`, list not null, elements not blank (`staging/dto/SetPermissionsRequest.java:9`). Replaces the set exactly (`StagingUserService:116-124`). Unknown id: 404 |
| `POST /api/v1/staging/sso-clients` | See section 7 |

- **User shape** (`staging/dto/StagingUserResponse.java:6-14`): `{id: number, email, firstName, lastName, role, permissions: [names in bit order], student: boolean}`. No password, no student detail.
- **Create body** (`staging/dto/CreateStagingUserRequest.java:21-38`):

| Field | Rule |
|---|---|
| `email` | not blank, an email, at most 254; stripped and lower-cased (`StagingUserService:64`) |
| `password` | 8 to 72 characters |
| `firstName`, `lastName` | not blank, at most 100 |
| `role` | not blank; stripped and upper-cased |
| `permissions` | optional; null means the role's defaults, `[]` means none |
| `student` | optional, only for role `STUDENT` |
| `student.facultyNumber` | not blank, at most 20 |
| `student.specialty` | not blank, at most 200 |
| `student.studyYear` | integer 1 to 6 |
| `student.status` | optional; blank or null means `ACTIVE`; stripped and upper-cased |

- **Errors** (all 400 `ApiError`):
  - Bad role: "role: must be one of [ADMIN, INSPECTOR, TEACHER, STUDENT]" (`:138-144`).
  - Unknown permission: "permissions: unknown permission 'X', known: [...]" (`:154-165`, `StagingToolsTests:145`).
  - `student` on another role: "student: only a user with role STUDENT has an academic record" (`:61-63`).
  - Bad status: "student.status: must be one of [...]" (`:146-152`).
- **409** for a duplicate email or a duplicate faculty number (`:65-67,102-114`, `StagingToolsTests:108`). Creation is one transaction, so a duplicate faculty number leaves no user behind.
- Created users can sign in with their password at `/api/v1/auth/login` (`StagingToolsTests:71`).

## 11. Behaviour pinned by tests, and gaps in `sso.md`

**Pinned by tests, not stated in `sso.md`:**
- A Bearer token of either kind does not authenticate `/oauth2/authorize` (`SsoFlowTests:339`).
- Sign-in through `sso-session` issues a new session id (`:110`). The registration answer has no `Set-Cookie` (about line 480).
- `continueUrl` is null with 200 when nothing waits (`:395`). 401 for an unknown user id (`:382`) and a missing Bearer (`:389`).
- A user disabled between code and exchange gets 400 `invalid_grant` and no `id_token` (`:365`).
- `?state=` on the logout redirect, and the bare target with no state in `any` mode (`:193`, `SsoAnyCallbackTests:135`).
- An unregistered callback, with state only, no nonce and scope `openid`, works in `any` mode (`SsoAnyCallbackTests:90-98`). Registration without `postLogoutRedirectUris` is accepted (`:158`).
- Host matching ignores case, and `*.x` covers nested subdomains (`SsoCallbackPolicyTests:20-37`).
- A logout-style save of a redeemed authorization still works, and cleanup keeps rows newer than the cutoff (`SsoAuthorizationStoreTests:87,103`).
- A session exactly at the 8-hour boundary is kept. An older one ends and the request goes on as anonymous (`SsoSessionMaxAgeFilterTests:34-44`).
- Staging: 10 permissions, and `roleDefaults.ADMIN[0] == STAFF_CREATE` (`StagingToolsTests:176`). Duplicate faculty number gives 409 (`:108`).

**Where `sso.md` and the Java code disagree:**
1. `docs/sso.md:357` says `clientId` is "2 to 63 lower-case letters, digits or dashes". The first character also cannot be a dash (`RegisterSsoClientRequest.java:20`).
2. `docs/sso.md:231` says logout data is kept 24 hours. The hourly sweep keeps it 24 to about 25 hours (`SsoAuthorizationCleanup:24,36`).
3. `db/migration/V4__sso.sql:84` names the cookie `UNICLOUD_SSO`. The code uses `__Host-UNICLOUD_SSO` (`config:74`). A stale comment with no effect.
4. `docs/sso.md:77-84` lists six endpoints. The library also serves revocation, introspection and device endpoints, and probably advertises them (Unverified, Spring default).
5. `docs/sso.md:112` (`invalid_request` without PKCE) is pinned by `SsoFlowTests:302`. `docs/sso.md:319` (`invalid_scope`) is not tested.
6. `docs/sso.md:174` shows errors as only `{"error": "..."}`. The library normally adds `error_description` (Unverified, Spring default).
7. `docs/sso.md:155` shows `expires_in: 1799`. That is library rounding, not pinned.
8. `docs/sso.md:203-204` lists `auth_time`. No test or repo code shows it in the token.
9. No ID-token lifetime is set. `docs/sso.md:323` ("expires after 30 minutes") is about the access token. The ID-token lifetime is Unverified (Spring default).

## Changes in the Rust portal

The Rust portal keeps everything above that `sso.md` sections 1 to 10 promise to module teams. These changes are deliberate ([ADR 0002](adr/0002-grow-the-sso-contract.md), [ADR 0003](adr/0003-one-cookie-session.md)):

- **New role `DEAN`.** `role` gains a fifth value. Module teams are told that `role` may gain values and that an unknown role means no access.
- **New permission `STUDENT_PROFILE_READ_FACULTY` at bit 10.** Bits 0 to 9 keep their names. Dean defaults: `STUDENT_PROFILE_READ_FACULTY`, `COURSE_OFFERING_READ`.
- **New claim `faculty`** in the `unicloud` scope: `{"code": "FCST", "name": "..."}`. Present for every user who has a faculty, absent for admins.
- **Directory API** under `/api/v1/directory`, read-only. A module calls it with an access token from the `client_credentials` grant that carries the `directory` scope. A module gets that scope at registration. Discovery advertises `client_credentials`.
- **The portal's own JWT is removed**, and so are `POST /api/v1/auth/login`, `POST /api/v1/auth/sso-session` and `DELETE /api/v1/auth/sso-session`. The portal UI and SSO share one cookie session, `__Host-UNICLOUD_SSO`. The 8-hour limit counts from the moment the user typed the password.
- **Admins register modules on a portal page**, signed in with the cookie session and protected against CSRF. `POST /api/v1/sso/clients` with a Bearer token is removed. The staging API `POST /api/v1/staging/sso-clients` stays as it is.
- **Fresh database.** Spring's `oauth2_*` and `SPRING_SESSION*` tables are not migrated. Module teams register again on staging. The issuer URL stays the same.
- **Spelling.** New wire names use US "enrollment", as the existing permission names do.
- **Unverified items are decided, not inherited.** Where this file says Unverified (Spring default), the Rust portal sends what the recording of the Java staging shows, unless the recording contradicts `sso.md`, in which case `sso.md` wins.

## Recorded from the Java staging (2026-11-20)

The file `tests/golden/java-staging-2026-11-20.json` holds a recording of the live Java staging. Secrets, tokens and codes are redacted. It settles these "Unverified" items:

- **Discovery** advertises the Spring defaults: `client_secret_basic`, `client_secret_post`, `client_secret_jwt`, `private_key_jwt` and TLS methods; the grants `authorization_code`, `client_credentials`, `refresh_token` and token exchange; revocation and introspection endpoints; DPoP algorithms; and `scopes_supported: ["openid"]`. Only part of this works for a registered module. The Rust portal advertises only what it serves.
- **JWKS**: one key with `kty`, `e`, `kid` and `n`. There is no `use` and no `alg`.
- **Token response**: `access_token`, `scope` (space separated, any order), `id_token`, `token_type` `"Bearer"`, `expires_in` `1799`.
- **ID token**: the header is `{"kid", "alg": "RS256"}`, with no `typ`. Claims: `iss`, `sub` (string), `aud` (**a single string**, not an array), `azp`, `exp`, `iat`, `auth_time`, `nonce` (if sent), `sid`, `jti`, plus the user claims for the granted scopes.
- **Access token**: `iss`, `sub`, `aud` (string), `nbf`, `iat`, `exp`, `jti`, and `scope` as a **JSON array**.
- **`/userinfo`**: GET and POST both work. Without a token, or with an ID token, a revoked token or a portal token, the answer is 401 `{"error": "invalid_token"}`.
- **Token errors**:
  - A replayed code or a wrong verifier gives 400 `{"error": "invalid_grant"}`.
  - A wrong secret, or `client_secret_post`, gives 401 `{"error": "invalid_client"}`.
  - `client_credentials` for a client without that grant gives 400 `unauthorized_client`.
  - An unknown grant gives 400 with `error`, `error_description` and `error_uri`.
- **Authorize errors that redirect** carry `error`, `error_description`, `error_uri` and `state`. For example, missing PKCE gives `error_description=OAuth 2.0 Parameter: code_challenge`, and `plain` PKCE names `code_challenge_method`.
- **Authorize errors that do not redirect**: an unknown client, or `response_type=token`, gave **401 with an empty body**. `docs/sso.md` promises a plain 400 page. The Rust portal follows the doc: a 400 page.
- **Cookie**: `Path=/; Secure; HttpOnly; SameSite=Lax`. Logout clears it with `Max-Age=0`.
- **`continueUrl`** ends with a stray `&continue` that Spring appends. It has no effect.
