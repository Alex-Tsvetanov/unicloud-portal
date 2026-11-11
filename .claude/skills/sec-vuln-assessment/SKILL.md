---
name: sec-vuln-assessment
description: Run the UniCloud vulnerability assessment and maintain the vulnerability register - SQL injection, XSS and output encoding, security misconfiguration (Spring Boot, actuator, CORS, headers, error pages, TLS), vulnerable dependencies and container image, plus import of security-audit and CI scanner results. Use for "vulnerability assessment", "check for SQL injection / XSS", "misconfiguration", "dependency vulnerabilities", "is it safe to merge", or after code, pom.xml or Dockerfile changes.
---

# Vulnerability assessment

Maintains `docs/security/VULNERABILITY-ASSESSMENT.md`, the register of every finding (V-xx). Read `AGENTS.md` first; the evidence bar there decides what is a finding.

## 1. SQL injection

```bash
grep -rnE 'jdbc\.(query|update|queryFor|execute|batchUpdate)|createNativeQuery|createQuery|@Query|EntityManager|Statement|prepareStatement' src/main/java
grep -rnE '"\s*\+\s*[a-zA-Z_]|\+\s*"\s*(WHERE|AND|OR|ORDER|LIMIT)|String\.format\(.*(SELECT|INSERT|UPDATE|DELETE)|formatted\(' src/main/java
```

- Every value goes through `?`/named parameters. Concatenated identifiers (sort column, direction) must come from an allowlist, never the request.
- `@Query` JPQL with `:param` is fine; `nativeQuery = true` with concatenation is not.
- `LIKE` searches: parameterized, and `%`/`_` escaped if user input shouldn't act as wildcards.
- Flyway migrations are trusted input; check only that no migration inserts real credentials or personal data.

## 2. XSS and output handling

- API responses: `@RestController` returning objects → JSON with `application/json`. Look for `String` returns containing user data, `produces = "text/html"`, `ResponseEntity<String>` with HTML, and templates (`src/main/resources/templates`, Thymeleaf `th:utext`, any frontend).
- Stored XSS sources: names, specialty, course titles. If a frontend renders them, it must use text binding (`textContent`, framework escaping), never `innerHTML`/`v-html`/`dangerouslySetInnerHTML`.
- Error responses must not reflect input unescaped (`server.error.include-message`, custom handlers).

## 3. Misconfiguration

Check `application*.properties`/`.yml`, config classes and `Dockerfile`:

- Actuator: if present, only `health` exposed (`management.endpoints.web.exposure.include=health`); `env`, `heapdump`, `configprops`, `loggers`, `mappings` never exposed.
- Errors: `server.error.include-stacktrace=never`, `include-message`/`include-binding-errors` not `always` in production.
- Security headers (via Spring Security defaults or a filter): HSTS, `X-Content-Type-Options: nosniff`, `X-Frame-Options`/`frame-ancestors`, a CSP if HTML is served, `Referrer-Policy`. Note that `ServedByFilter` exposes the replica hostname by design (README); it is info-level only.
- CORS: no `allowedOrigins("*")` with credentials; explicit UniCloud origins.
- Info leakage at `/` and `/health`: only service name, replica and version (documented as intended).
- DB connection: `sslMode=VERIFY_IDENTITY` vs `REQUIRED` (REQUIRED encrypts but doesn't verify the server certificate); `allowPublicKeyRetrieval`, `allowLoadLocalInfile`, `autoDeserialize` must not be `true`.
- Container: non-root (`USER 1000` present), minimal JRE base, no secrets in `ARG`/`ENV`, `.dockerignore` excludes `.git`, `.env`.
- `spring.jpa.open-in-view=false`, `ddl-auto=validate` (already in place: list as passed checks).

## 4. Dependencies and image

Prefer results from the CI `security` workflow (Trivy filesystem and image scan, CodeQL, dependency review) via `gh` as described in `sec-sweep`. If none are available and the tool is already installed locally, you may run, offline-safe:

```bash
trivy fs --scanners vuln,secret,misconfig --severity HIGH,CRITICAL .
./mvnw -q dependency:tree -DoutputType=text   # to see what pulls a vulnerable artifact
```

Do not install tools as part of the assessment. A CVE counts as a finding only if the vulnerable code path is reachable in this app; otherwise record it as `hardening` with "upgrade" as the fix. Spring Boot's managed versions are upgraded by bumping the parent version in `pom.xml`.

## 5. Other classes worth a pass every time

Mass assignment (see `sec-access-control`), race conditions in enrollment (README locking rule; cancel/re-enroll path takes the same lock), unbounded list endpoints and pagination (DoS at 08:00), file upload (none yet), SSRF (any outbound HTTP call built from input), deserialization (`ObjectInputStream`, polymorphic Jackson typing), open redirects after login.

## 6. Deep hunt

For `--deep` sweeps, run the third-party `security-audit` skill in full audit mode with its default output directory outside the repo, then import its `confirmed` and `needs_validation` records here, carrying their fingerprints. Its `rejected` records are not imported, but a one-line count goes in the report.

## 7. Register format

`docs/security/VULNERABILITY-ASSESSMENT.md` has a header (date, SHA), a methodology paragraph, then one table and one detail section per item, using the finding shape from `sec-sweep`. Statuses move forward: `needs-validation → confirmed → fixed`, or `→ accepted-risk` (with who accepted it and until when), or `→ retired` (code removed). Hardening notes live in their own table at the end. Respect the disclosure rule.
