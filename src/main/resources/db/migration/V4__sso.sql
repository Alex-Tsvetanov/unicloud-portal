-- Single sign-on for the other UniCloud modules (docs/sso.md).
-- The first three tables are the schemas Spring's OAuth 2.0 Authorization Server and Spring
-- Session JDBC expect, adapted to MySQL. Their table and column names must not change: the
-- libraries' own SQL refers to them.

-- The modules ("clients") allowed to log users in through the portal.
-- client_secret holds a BCrypt hash, never the secret itself.
CREATE TABLE oauth2_registered_client (
    id                            VARCHAR(100)  NOT NULL,
    client_id                     VARCHAR(100)  NOT NULL,
    client_id_issued_at           TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    client_secret                 VARCHAR(200)  DEFAULT NULL,
    client_secret_expires_at      TIMESTAMP     NULL DEFAULT NULL,
    client_name                   VARCHAR(200)  NOT NULL,
    client_authentication_methods VARCHAR(1000) NOT NULL,
    authorization_grant_types     VARCHAR(1000) NOT NULL,
    redirect_uris                 VARCHAR(1000) DEFAULT NULL,
    post_logout_redirect_uris     VARCHAR(1000) DEFAULT NULL,
    scopes                        VARCHAR(1000) NOT NULL,
    client_settings               VARCHAR(2000) NOT NULL,
    token_settings                VARCHAR(2000) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE KEY uq_oauth2_registered_client_client_id (client_id)
);

-- One row per login through SSO: the authorization code, then the tokens issued for it.
-- MEDIUMBLOB instead of BLOB: BLOB stops at 64 KB. The prefix indexes serve the lookups by
-- code and token value. Rows older than a day are deleted by SsoAuthorizationCleanup.
CREATE TABLE oauth2_authorization (
    id                            VARCHAR(100)  NOT NULL,
    registered_client_id          VARCHAR(100)  NOT NULL,
    principal_name                VARCHAR(200)  NOT NULL,
    authorization_grant_type      VARCHAR(100)  NOT NULL,
    authorized_scopes             VARCHAR(1000) DEFAULT NULL,
    attributes                    MEDIUMBLOB    DEFAULT NULL,
    state                         VARCHAR(500)  DEFAULT NULL,
    authorization_code_value      MEDIUMBLOB    DEFAULT NULL,
    authorization_code_issued_at  TIMESTAMP     NULL DEFAULT NULL,
    authorization_code_expires_at TIMESTAMP     NULL DEFAULT NULL,
    authorization_code_metadata   MEDIUMBLOB    DEFAULT NULL,
    access_token_value            MEDIUMBLOB    DEFAULT NULL,
    access_token_issued_at        TIMESTAMP     NULL DEFAULT NULL,
    access_token_expires_at       TIMESTAMP     NULL DEFAULT NULL,
    access_token_metadata         MEDIUMBLOB    DEFAULT NULL,
    access_token_type             VARCHAR(100)  DEFAULT NULL,
    access_token_scopes           VARCHAR(1000) DEFAULT NULL,
    oidc_id_token_value           MEDIUMBLOB    DEFAULT NULL,
    oidc_id_token_issued_at       TIMESTAMP     NULL DEFAULT NULL,
    oidc_id_token_expires_at      TIMESTAMP     NULL DEFAULT NULL,
    oidc_id_token_metadata        MEDIUMBLOB    DEFAULT NULL,
    refresh_token_value           MEDIUMBLOB    DEFAULT NULL,
    refresh_token_issued_at       TIMESTAMP     NULL DEFAULT NULL,
    refresh_token_expires_at      TIMESTAMP     NULL DEFAULT NULL,
    refresh_token_metadata        MEDIUMBLOB    DEFAULT NULL,
    user_code_value               MEDIUMBLOB    DEFAULT NULL,
    user_code_issued_at           TIMESTAMP     NULL DEFAULT NULL,
    user_code_expires_at          TIMESTAMP     NULL DEFAULT NULL,
    user_code_metadata            MEDIUMBLOB    DEFAULT NULL,
    device_code_value             MEDIUMBLOB    DEFAULT NULL,
    device_code_issued_at         TIMESTAMP     NULL DEFAULT NULL,
    device_code_expires_at        TIMESTAMP     NULL DEFAULT NULL,
    device_code_metadata          MEDIUMBLOB    DEFAULT NULL,
    -- Ours, not the library's: set once, atomically, when the code is exchanged for tokens.
    -- Two replicas exchanging the same code at the same moment cannot both succeed.
    -- See SingleUseCodeAuthorizationService.
    code_redeemed                 BOOLEAN       NOT NULL DEFAULT FALSE,
    created_at                    TIMESTAMP     NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    KEY ix_oauth2_authorization_code (authorization_code_value(255)),
    KEY ix_oauth2_authorization_access_token (access_token_value(255)),
    KEY ix_oauth2_authorization_id_token (oidc_id_token_value(255)),
    KEY ix_oauth2_authorization_created_at (created_at)
);

-- Consent is not asked for (the modules are first-party), but the library's JDBC consent
-- store is the only one that works across replicas, so the table exists.
CREATE TABLE oauth2_authorization_consent (
    registered_client_id VARCHAR(100)  NOT NULL,
    principal_name       VARCHAR(200)  NOT NULL,
    authorities          VARCHAR(1000) NOT NULL,
    PRIMARY KEY (registered_client_id, principal_name)
);

-- The portal's SSO browser session (cookie UNICLOUD_SSO), shared by every replica.
CREATE TABLE SPRING_SESSION (
    PRIMARY_ID            CHAR(36)     NOT NULL,
    SESSION_ID            CHAR(36)     NOT NULL,
    CREATION_TIME         BIGINT       NOT NULL,
    LAST_ACCESS_TIME      BIGINT       NOT NULL,
    MAX_INACTIVE_INTERVAL INT          NOT NULL,
    EXPIRY_TIME           BIGINT       NOT NULL,
    PRINCIPAL_NAME        VARCHAR(100),
    CONSTRAINT SPRING_SESSION_PK PRIMARY KEY (PRIMARY_ID)
) ENGINE=InnoDB ROW_FORMAT=DYNAMIC;

CREATE UNIQUE INDEX SPRING_SESSION_IX1 ON SPRING_SESSION (SESSION_ID);
CREATE INDEX SPRING_SESSION_IX2 ON SPRING_SESSION (EXPIRY_TIME);
CREATE INDEX SPRING_SESSION_IX3 ON SPRING_SESSION (PRINCIPAL_NAME);

CREATE TABLE SPRING_SESSION_ATTRIBUTES (
    SESSION_PRIMARY_ID CHAR(36)     NOT NULL,
    ATTRIBUTE_NAME     VARCHAR(200) NOT NULL,
    ATTRIBUTE_BYTES    BLOB         NOT NULL,
    CONSTRAINT SPRING_SESSION_ATTRIBUTES_PK PRIMARY KEY (SESSION_PRIMARY_ID, ATTRIBUTE_NAME),
    CONSTRAINT SPRING_SESSION_ATTRIBUTES_FK FOREIGN KEY (SESSION_PRIMARY_ID)
        REFERENCES SPRING_SESSION (PRIMARY_ID) ON DELETE CASCADE
) ENGINE=InnoDB ROW_FORMAT=DYNAMIC;
