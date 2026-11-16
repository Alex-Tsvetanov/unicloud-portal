package bg.tusofia.unicloud.portal.auth.application;

import bg.tusofia.unicloud.portal.user.domain.Permission;
import bg.tusofia.unicloud.portal.user.domain.User;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.security.oauth2.jose.jws.SignatureAlgorithm;
import org.springframework.security.oauth2.jwt.JwsHeader;
import org.springframework.security.oauth2.jwt.JwtClaimsSet;
import org.springframework.security.oauth2.jwt.JwtEncoder;
import org.springframework.security.oauth2.jwt.JwtEncoderParameters;
import org.springframework.stereotype.Service;

import java.time.Duration;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.List;

@Service
public class JwtAccessTokenIssuer implements AccessTokenIssuer {

    private final JwtEncoder jwtEncoder;
    private final String issuer;
    private final String audience;
    private final long ttlMinutes;

    public JwtAccessTokenIssuer(
            JwtEncoder jwtEncoder,
            @Value("${app.jwt.issuer}") String issuer,
            @Value("${app.jwt.audience}") String audience,
            @Value("${app.jwt.ttl-minutes}") long ttlMinutes) {

        this.jwtEncoder = jwtEncoder;
        this.issuer = issuer;
        this.audience = audience;
        this.ttlMinutes = ttlMinutes;
    }

    @Override
    public AccessToken issue(User user) {
        Instant now = Instant.now();
        Instant expiresAt = now.plus(
                ttlMinutes, ChronoUnit.MINUTES);

        JwtClaimsSet claims = JwtClaimsSet.builder()
                .issuer(issuer)
                .subject(user.getId().toString())
                .audience(List.of(audience))
                .issuedAt(now)
                .expiresAt(expiresAt)
                .claim(
                        "role",
                        user.getRole().name()
                ).claim("permissions",
                        Permission.fromMask(user.getPermissionsMask()))
                .build();

        JwsHeader header = JwsHeader
                .with(SignatureAlgorithm.RS256)
                .build();

        String token = jwtEncoder.encode(
                JwtEncoderParameters.from(header, claims)
        ).getTokenValue();

        return new AccessToken(
                token,
                Duration.between(now, expiresAt).toSeconds()
        );
    }
}
