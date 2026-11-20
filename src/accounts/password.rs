//! Password hashing with Argon2id. Hashing runs on a blocking thread, a few at a time, so a burst of
//! sign-ins cannot starve the server.

use std::sync::OnceLock;

use argon2::Argon2;
use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use tokio::sync::Semaphore;

pub const MIN_LEN: usize = 8;
pub const MAX_LEN: usize = 72;
/// Argon2 runs at most this many times at once per replica
const PARALLEL_HASHES: usize = 4;

fn permits() -> &'static Semaphore {
    static S: OnceLock<Semaphore> = OnceLock::new();
    S.get_or_init(|| Semaphore::new(PARALLEL_HASHES))
}

fn hash_blocking(password: &str) -> anyhow::Result<String> {
    let mut salt = [0u8; 16];
    getrandom::fill(&mut salt)?;
    Argon2::default()
        .hash_password_with_salt(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

pub async fn hash(password: String) -> anyhow::Result<String> {
    let _permit = permits().acquire().await?;
    tokio::task::spawn_blocking(move || hash_blocking(&password)).await?
}

/// False for a wrong password or a hash this portal cannot read
pub async fn verify(password: String, stored: String) -> bool {
    let Ok(_permit) = permits().acquire().await else { return false };
    tokio::task::spawn_blocking(move || {
        PasswordHash::new(&stored)
            .map(|h| Argon2::default().verify_password(password.as_bytes(), &h).is_ok())
            .unwrap_or(false)
    })
    .await
    .unwrap_or(false)
}

/// Spends the same time as a real check, for an email that does not exist (no account probing)
pub async fn verify_dummy(password: String) {
    static DUMMY: OnceLock<String> = OnceLock::new();
    let stored = match DUMMY.get() {
        Some(h) => h.clone(),
        None => match tokio::task::spawn_blocking(|| hash_blocking("unicloud-dummy-password")).await {
            Ok(Ok(h)) => DUMMY.get_or_init(|| h).clone(),
            _ => return,
        },
    };
    let _ = verify(password, stored).await;
}

pub fn check_strength(p: &str) -> Result<(), &'static str> {
    if p.chars().count() < MIN_LEN {
        return Err("Паролата трябва да е поне 8 знака.");
    }
    if p.chars().count() > MAX_LEN {
        return Err("Паролата трябва да е най-много 72 знака.");
    }
    Ok(())
}
