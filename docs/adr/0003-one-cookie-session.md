# One cookie session for the portal UI and SSO

Status: accepted (2026-11-20)

The Java portal had two logins: a Bearer JWT kept in each tab's `sessionStorage` for the portal's own API, and a `__Host-UNICLOUD_SSO` cookie for single sign-on, bridged by `POST /api/v1/auth/sso-session`. With a server-rendered UI there is no browser code that needs a token. So the portal keeps only the cookie session, stored in PostgreSQL so every replica sees it. `/api/v1/auth/login`, `/api/v1/auth/sso-session` and the portal's own JWT (`aud=unicloud-api`) are removed.

Modules are not affected: sections 1 to 10 of `docs/sso.md` never mention these routes. Signing out of the portal still ends the SSO session. The old note that "losing the portal token in one tab does not end the SSO session" no longer applies, because there is no per-tab token.

Forms that change state need CSRF protection. A Bearer header used to give that protection for free, and a cookie does not.
