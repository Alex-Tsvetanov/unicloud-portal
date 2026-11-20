//! Password hashing with Argon2id. Hashing runs on a blocking thread so it never stalls the server.

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};

pub const MIN_LEN: usize = 8;
pub const MAX_LEN: usize = 72;

pub async fn hash(password: String) -> anyhow::Result<String> {
    tokio::task::spawn_blocking(move || {
        let mut salt_bytes = [0u8; 16];
        getrandom::fill(&mut salt_bytes)?;
        Argon2::default()
            .hash_password_with_salt(password.as_bytes(), &salt_bytes)
            .map(|h| h.to_string())
            .map_err(|e| anyhow::anyhow!("{e}"))
    })
    .await?
}

/// False for a wrong password or a hash this portal cannot read
pub async fn verify(password: String, stored: String) -> bool {
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&stored)
            .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

/// A hash that matches no password, for users who must not sign in (and for timing equality)
pub const NO_LOGIN: &str = "!";

pub fn check_strength(p: &str) -> Result<(), &'static str> {
    if p.chars().count() < MIN_LEN {
        return Err("Паролата трябва да е поне 8 знака.");
    }
    if p.chars().count() > MAX_LEN {
        return Err("Паролата трябва да е най-много 72 знака.");
    }
    Ok(())
}
