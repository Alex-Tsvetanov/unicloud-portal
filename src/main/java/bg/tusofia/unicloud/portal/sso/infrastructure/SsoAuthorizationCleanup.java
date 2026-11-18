package bg.tusofia.unicloud.portal.sso.infrastructure;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.jdbc.core.JdbcOperations;
import org.springframework.scheduling.annotation.Scheduled;
import org.springframework.stereotype.Component;

import java.sql.Timestamp;
import java.time.Duration;
import java.time.Instant;

/**
 * Deletes old SSO logins from oauth2_authorization. The library never deletes them, and the
 * table would otherwise grow with every login.
 *
 * <p>A row is kept for a day. Tokens live 30 minutes, but a module may still send an older ID
 * token to /connect/logout, which needs the row to find the user. Every replica runs this; the
 * DELETE is harmless when another replica got there first.
 */
@Component
public class SsoAuthorizationCleanup {

    static final Duration RETENTION = Duration.ofDays(1);

    private static final Logger log = LoggerFactory.getLogger(SsoAuthorizationCleanup.class);

    private static final String DELETE_OLD = "DELETE FROM oauth2_authorization WHERE created_at < ?";

    private final JdbcOperations jdbcOperations;

    public SsoAuthorizationCleanup(JdbcOperations jdbcOperations) {
        this.jdbcOperations = jdbcOperations;
    }

    @Scheduled(fixedDelayString = "PT1H", initialDelayString = "PT5M")
    public void scheduledCleanup() {
        deleteOlderThan(Instant.now().minus(RETENTION));
    }

    /** Deletes the logins created before {@code cutoff}; returns how many. */
    public int deleteOlderThan(Instant cutoff) {
        int deleted = jdbcOperations.update(DELETE_OLD, Timestamp.from(cutoff));
        if (deleted > 0) {
            log.info("Deleted {} SSO logins older than {}", deleted, cutoff);
        }
        return deleted;
    }
}
