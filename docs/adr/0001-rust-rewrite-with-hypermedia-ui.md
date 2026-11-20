# Rewrite the portal in Rust, with a hand-built OIDC provider and a hypermedia UI

Status: accepted (2026-11-20)

We are replacing the Spring Boot portal with one Rust service. It uses axum, sqlx on PostgreSQL, and server-rendered HTML: components are Maud functions, and htmx (2.0.x) swaps fragments. There is no JavaScript build step. The Java code is deleted once the Rust build passes the SSO conformance suite. The suite is ported from the Java tests and checked against a recording of the Java staging deployment.

The OpenID Connect provider is written by us on top of `jsonwebtoken`, not taken from a library. In November 2026 no Rust crate covered the OIDC layer we must keep compatible with `docs/sso.md`: discovery, id_token, userinfo and RP-initiated logout. `oxide-auth` is OAuth2 only and stale, `oauth-as` states it is not an OP, and `authkestra-op` is pre-1.0 and changes weekly. Our surface is small and fixed (about eight endpoints), so owning it costs less than adapting a framework. Do not "replace it with a library" without checking that the library passes the same conformance suite.

## Consequences

- PostgreSQL is the only database. The MySQL migrations and the Azure MySQL server go away, and production moves to Azure PostgreSQL Flexible Server.
- The cutover starts with a fresh database, on staging and on production. No data is copied from the Azure MySQL server. Spring's `oauth2_*` tables and their Java-typed JSON are not migrated. Module teams register their modules again, and the issuer URL stays the same.
