//! Small cryptographic helpers: randomness, hashing, encodings, constant-time comparison.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// `n` random bytes, URL-safe base64 without padding (32 bytes give 43 characters)
pub fn random_token(n: usize) -> String {
    let mut buf = vec![0u8; n];
    getrandom::fill(&mut buf).expect("the operating system's random source failed");
    URL_SAFE_NO_PAD.encode(buf)
}

pub fn sha256_hex(s: &str) -> String {
    Sha256::digest(s.as_bytes()).iter().map(|b| format!("{b:02x}")).collect()
}

pub fn b64url(bytes: &[u8]) -> String {
    URL_SAFE_NO_PAD.encode(bytes)
}

pub fn ct_eq(a: &str, b: &str) -> bool {
    a.len() == b.len() && bool::from(a.as_bytes().ct_eq(b.as_bytes()))
}

/// PKCE S256 (RFC 7636 section 4.6): BASE64URL(SHA256(ASCII(code_verifier))) == code_challenge
pub fn pkce_s256_matches(verifier: &str, challenge: &str) -> bool {
    let ok_len = (43..=128).contains(&verifier.len());
    let ok_chars = verifier.bytes().all(|c| c.is_ascii_alphanumeric() || b"-._~".contains(&c));
    ok_len && ok_chars && ct_eq(&b64url(&Sha256::digest(verifier.as_bytes())), challenge)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pkce_matches_rfc_7636_appendix_b() {
        assert!(pkce_s256_matches(
            "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk",
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        ));
        assert!(!pkce_s256_matches(
            "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXX",
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        ));
    }

    #[test]
    fn random_token_of_32_bytes_has_43_chars() {
        assert_eq!(random_token(32).len(), 43);
    }
}
