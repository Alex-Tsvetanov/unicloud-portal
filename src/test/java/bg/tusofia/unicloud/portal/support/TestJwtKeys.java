package bg.tusofia.unicloud.portal.support;

import org.springframework.test.context.DynamicPropertyRegistry;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.GeneralSecurityException;
import java.security.KeyPair;
import java.security.KeyPairGenerator;
import java.util.Base64;

/**
 * The app signs and checks JWTs with an RSA key pair read from {@code ./secrets} (never
 * committed), so a Spring test context cannot start without one. Tests use a throw-away pair
 * generated here: nothing to set up locally and no key material in the repository.
 *
 * <pre>
 * &#64;DynamicPropertySource
 * static void jwtKeys(DynamicPropertyRegistry registry) {
 *     TestJwtKeys.register(registry);
 * }
 * </pre>
 */
public final class TestJwtKeys {

    private TestJwtKeys() {
    }

    /** Points {@code app.jwt.public-key} and {@code app.jwt.private-key} at a fresh key pair. */
    public static void register(DynamicPropertyRegistry registry) {
        Path dir = writeKeyPair();
        registry.add("app.jwt.public-key", () -> dir.resolve("public.pem").toUri().toString());
        registry.add("app.jwt.private-key", () -> dir.resolve("private.pem").toUri().toString());
    }

    private static Path writeKeyPair() {
        try {
            KeyPairGenerator generator = KeyPairGenerator.getInstance("RSA");
            generator.initialize(2048);
            KeyPair pair = generator.generateKeyPair();

            Path dir = Files.createTempDirectory("unicloud-test-jwt-");
            Path publicKey = dir.resolve("public.pem");
            Path privateKey = dir.resolve("private.pem");
            // The encodings Spring Security's RsaKeyConverters read: X.509 SubjectPublicKeyInfo
            // for the public key and PKCS#8 for the private one, the same as openssl produces.
            Files.writeString(publicKey, pem("PUBLIC KEY", pair.getPublic().getEncoded()));
            Files.writeString(privateKey, pem("PRIVATE KEY", pair.getPrivate().getEncoded()));

            // deleteOnExit runs in reverse registration order, so the files go before the directory
            dir.toFile().deleteOnExit();
            publicKey.toFile().deleteOnExit();
            privateKey.toFile().deleteOnExit();
            return dir;
        } catch (GeneralSecurityException | IOException e) {
            throw new IllegalStateException("Could not create the JWT key pair for the tests", e);
        }
    }

    private static String pem(String type, byte[] der) {
        String body = Base64.getMimeEncoder(64, "\n".getBytes(StandardCharsets.US_ASCII))
                .encodeToString(der);
        return "-----BEGIN " + type + "-----\n" + body + "\n-----END " + type + "-----\n";
    }
}
