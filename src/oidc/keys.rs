//! The token signing key: RS256, `kid` = RFC 7638 thumbprint, published at `/oauth2/jwks`.

use anyhow::Context;
use jsonwebtoken::jwk::{AlgorithmParameters, Jwk};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::crypto::b64url;

pub struct SigningKeys {
    encoding: EncodingKey,
    decoding: DecodingKey,
    pub kid: String,
    n: String,
    e: String,
}

impl SigningKeys {
    pub fn from_pem(pem: &str) -> anyhow::Result<Self> {
        let encoding = EncodingKey::from_rsa_pem(pem.as_bytes()).context("APP_SIGNING_KEY_PEM is not an RSA private key in PEM")?;
        let jwk = Jwk::from_encoding_key(&encoding, Algorithm::RS256).context("cannot derive the public key")?;
        let AlgorithmParameters::RSA(rsa) = &jwk.algorithm else {
            anyhow::bail!("APP_SIGNING_KEY_PEM must be an RSA key");
        };
        let (n, e) = (rsa.n.clone(), rsa.e.clone());
        let decoding = DecodingKey::from_rsa_components(&n, &e).context("cannot build the verification key")?;
        let kid = rfc7638_thumbprint(&n, &e);
        Ok(Self {
            encoding,
            decoding,
            kid,
            n,
            e,
        })
    }

    pub fn sign(&self, claims: &Value) -> anyhow::Result<String> {
        let mut header = Header::new(Algorithm::RS256);
        header.typ = None;
        header.kid = Some(self.kid.clone());
        Ok(jsonwebtoken::encode(&header, claims, &self.encoding)?)
    }

    pub fn decoding(&self) -> &DecodingKey {
        &self.decoding
    }

    /// The JWKS document: the public key only. Same members as the Java portal (no `use`, no `alg`).
    pub fn jwks(&self) -> Value {
        json!({ "keys": [{ "kty": "RSA", "e": self.e, "kid": self.kid, "n": self.n }] })
    }
}

/// RFC 7638: SHA-256 over `{"e":..,"kty":"RSA","n":..}` (members sorted, no whitespace), base64url
pub fn rfc7638_thumbprint(n: &str, e: &str) -> String {
    let canonical = format!(r#"{{"e":"{e}","kty":"RSA","n":"{n}"}}"#);
    b64url(&Sha256::digest(canonical.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The example key of RFC 7638 section 3.1
    #[test]
    fn thumbprint_matches_rfc_7638_example() {
        let n = "0vx7agoebGcQSuuPiLJXZptN9nndrQmbXEps2aiAFbWhM78LhWx4cbbfAAtVT86zwu1RK7aPFFxuhDR1L6tSoc_BJECPebWKRXjBZCiFV4n3oknjhMstn64tZ_2W-5JsGY4Hc5n9yBXArwl93lqt7_RN5w6Cf0h4QyQ5v-65YGjQR0_FDW2QvzqY368QQMicAtaSqzs8KJZgnYb9c7d0zgdAZHzu6qMQvRL5hajrn1n91CbOpbISD08qNLyrdkt-bFTWhAI4vMQFh6WeZu0fM4lFd2NcRwr3XPksINHaQ-G_xBniIqbw0Ls1jF44-csFCur-kEgU8awapJzKnqDKgw";
        assert_eq!(rfc7638_thumbprint(n, "AQAB"), "NzbLsXh8uDCcd-6MNwXF4W_7noWXFZAfHkxZsRGC9Xs");
    }
}
