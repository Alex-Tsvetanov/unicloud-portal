package bg.tusofia.unicloud.portal;

/** The replica's name. In Azure Container Apps the HOSTNAME variable holds it. */
final class Replica {

    private Replica() {
    }

    static String name() {
        String host = System.getenv("HOSTNAME");
        return host == null || host.isBlank() ? "local" : host;
    }
}
