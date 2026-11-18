package bg.tusofia.unicloud.portal.sso;

import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.security.NoSuchAlgorithmException;
import java.util.Base64;
import java.util.UUID;

/** A PKCE pair, as a module makes it: the verifier it keeps, the S256 challenge it sends. */
record Pkce(String verifier, String challenge) {

    Pkce() {
        this(UUID.randomUUID() + "-" + UUID.randomUUID());
    }

    private Pkce(String verifier) {
        this(verifier, s256(verifier));
    }

    private static String s256(String verifier) {
        try {
            byte[] hash = MessageDigest.getInstance("SHA-256")
                    .digest(verifier.getBytes(StandardCharsets.US_ASCII));
            return Base64.getUrlEncoder().withoutPadding().encodeToString(hash);
        } catch (NoSuchAlgorithmException e) {
            throw new IllegalStateException(e);
        }
    }
}
