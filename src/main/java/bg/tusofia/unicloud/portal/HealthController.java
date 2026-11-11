package bg.tusofia.unicloud.portal;

import java.util.Map;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.web.bind.annotation.GetMapping;
import org.springframework.web.bind.annotation.RestController;

/**
 * Endpoints the pipeline and the demo rely on. Keep both when you add the real API.
 */
@RestController
public class HealthController {

    private final String version;
    private final String replica = Replica.name();

    public HealthController(@Value("${GIT_SHA:dev}") String version) {
        this.version = version;
    }

    /** Cheap on purpose: no database call. The deploy waits until this shows the new commit SHA. */
    @GetMapping("/health")
    public Map<String, String> health() {
        return Map.of("status", "ok", "version", version);
    }

    /** Shows which replica answered, for the load-balancing demo. */
    @GetMapping("/")
    public Map<String, String> info() {
        return Map.of("service", "unicloud-portal", "replica", replica, "version", version);
    }
}
