## ADDED Requirements

### Requirement: Health endpoint
`GET /health` SHALL answer 200 JSON with `status` `"UP"`, `service` `"unicloud-portal"` and `version` equal to the commit SHA the image was built from. It MUST NOT touch the database.

#### Scenario: Deploy check
- **WHEN** the deploy job polls `/health` after a release
- **THEN** the body contains the new commit SHA

#### Scenario: Database down
- **WHEN** the database is unreachable
- **THEN** `/health` still answers 200

### Requirement: Replica identity
Every response SHALL carry the header `X-Served-By` with the replica's name. `GET /` with `Accept: application/json` SHALL return `{service, version, replica}`. A browser request to `/` SHALL get the portal's home page or the sign-in page.

#### Scenario: Load spreading demo
- **WHEN** a client sends 20 requests to production
- **THEN** the `X-Served-By` values show at least two different replicas

### Requirement: Migrations on start
The portal SHALL apply pending database migrations at startup, before it accepts traffic. Migrations MUST be safe while an older replica still runs, so a column is renamed or dropped only in a later release than the code change.

#### Scenario: Fresh database
- **WHEN** the portal starts on an empty PostgreSQL database
- **THEN** it creates every table and the seed data, then serves requests

### Requirement: Graceful shutdown
On SIGTERM the portal SHALL stop accepting new connections and finish requests in flight for up to 20 seconds before it exits.

#### Scenario: Scale in during a request
- **WHEN** a replica gets SIGTERM while an enrollment request is running
- **THEN** that request completes with its normal answer

### Requirement: Database connection limits
Each replica SHALL use a connection pool whose size is set by configuration, with a default that keeps the maximum replica count within the database's connection limit.

#### Scenario: Ten replicas
- **WHEN** production runs 10 replicas with the default pool size
- **THEN** the total connections stay below the database server's limit

### Requirement: No single point of failure in production
Production SHALL run at least 2 replicas behind a load balancer, add replicas as concurrent requests grow, and keep at least 2 running during a deploy.

#### Scenario: One replica stops
- **WHEN** one production replica is stopped
- **THEN** requests keep succeeding through the other replicas

### Requirement: Staging on Render
Staging SHALL be defined by the Render Blueprint `render.yaml`: one free Docker web service and one free PostgreSQL database in Frankfurt, staging mode on, the issuer set to the service's external URL, and the signing key pair entered once as secrets.

#### Scenario: Blueprint sync
- **WHEN** the Blueprint is synced to an empty Render workspace
- **THEN** the portal starts with the staging tools and an issuer equal to its `https` address

### Requirement: Service levels are measured
The portal SHALL expose what is needed to measure the service levels in [docs/slo.md](../../../../../docs/slo.md): request outcome and latency per route group (sign-in, profile, enrollment, SSO token), logged per request as structured JSON with the replica name, without passwords, codes or tokens.

#### Scenario: Log line
- **WHEN** a student enrolls
- **THEN** the log has one JSON line with route group `enrollment`, status, duration and replica, and no token or password

### Requirement: Secrets from the environment
The database URL and the signing key pair SHALL come from environment variables. The portal MUST refuse to start without a signing key, and MUST never log a secret.

#### Scenario: Missing key
- **WHEN** the portal starts without the private key variable
- **THEN** it exits with a message naming the missing variable
