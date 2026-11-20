## ADDED Requirements

### Requirement: The portal app
The portal SHALL serve its user interface as a single-page app at `/app`. Any path under `/app/` that is not a built file SHALL return the app's `index.html`. A missing file under `/app/assets/` SHALL return 404.

#### Scenario: Deep link
- **WHEN** a browser opens `/app/courses`
- **THEN** the portal returns the app's `index.html` with `Cache-Control: no-cache`

#### Scenario: Missing asset
- **WHEN** a browser asks for `/app/assets/no-such-file.js`
- **THEN** the portal answers 404 and does not mark the answer as cacheable for a year

#### Scenario: Signed-in root
- **WHEN** a signed-in browser opens `/`
- **THEN** the portal redirects to `/app/`

### Requirement: Portal JSON API
The portal SHALL offer `/api/v1/portal/*` to its app. Every call SHALL use the session cookie. Every call that changes data SHALL carry the session's CSRF token in the `X-CSRF-Token` header. Every error SHALL be JSON `{error, message}` with a Bulgarian message. The API SHALL check the same permissions as the rest of the portal.

#### Scenario: No session
- **WHEN** a call to `/api/v1/portal/me` has no session
- **THEN** the portal answers 401 with `error` = `unauthenticated` and does not redirect

#### Scenario: First password not changed
- **WHEN** a user who must change the first password calls the API
- **THEN** the portal answers 403 with `error` = `password_change_required`

#### Scenario: Change after the session ended
- **WHEN** a browser whose session has ended posts an enrollment
- **THEN** the portal answers 401 with `error` = `unauthenticated`

#### Scenario: Missing CSRF header
- **WHEN** a signed-in student posts an enrollment without the right `X-CSRF-Token`
- **THEN** the portal answers 403 with `error` = `forbidden` and nothing changes

#### Scenario: Missing permission
- **WHEN** a student calls `/api/v1/portal/admin/staff`
- **THEN** the portal answers 403 with `error` = `forbidden`

#### Scenario: Refused enrollment
- **WHEN** a student enrolls in an offering they are already enrolled in
- **THEN** the portal answers 409 with `outcome` = `ALREADY_ENROLLED` and a Bulgarian message

#### Scenario: Module secret
- **WHEN** an admin registers a module through the API
- **THEN** the answer carries the secret once and `Cache-Control: no-store`

### Requirement: Return after sign-in
Sign-in SHALL accept a `next` path and return there after sign-in, only if the path starts with `/app/`, has no `//` and no backslash. Otherwise sign-in SHALL go to `/app/`. A waiting single sign-on login SHALL take priority.

#### Scenario: Return to the asked page
- **WHEN** a user signs in from `/login?next=/app/courses`
- **THEN** the portal redirects to `/app/courses`

#### Scenario: Another site
- **WHEN** a user signs in with `next` = `//evil.example/app/`
- **THEN** the portal redirects to `/app/`

### Requirement: Shared UI package
The portal SHALL serve `@unicloud/ui` as an npm tarball at `/ui/v1/unicloud-ui-<version>.tgz` and its stylesheet at `/ui/v1/unicloud.css`, readable from any origin. The package's components SHALL take data as props and SHALL NOT call the portal. A published version SHALL NOT change.

#### Scenario: Another team's page loads the stylesheet
- **WHEN** a page on another origin requests `/ui/v1/unicloud.css`
- **THEN** the answer carries `Access-Control-Allow-Origin: *`

#### Scenario: Another team installs the package
- **WHEN** a team downloads `/ui/v1/unicloud-ui-1.0.0.tgz` and runs `npm install` on the file
- **THEN** they get the components, their TypeScript types and the stylesheet
