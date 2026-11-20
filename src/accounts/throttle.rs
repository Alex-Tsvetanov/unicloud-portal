//! Slows down password guessing: after too many failed sign-ins for one email, that email is
//! locked for a while. Per replica, in memory. A limit per IP belongs at the ingress (docs/sso.md, "Not done yet").

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const MAX_FAILURES: u32 = 10;
const WINDOW: Duration = Duration::from_secs(15 * 60);
/// Stop the map from growing without bound under an attack with many emails
const MAX_TRACKED: usize = 100_000;

fn table() -> &'static Mutex<HashMap<String, (u32, Instant)>> {
    static T: OnceLock<Mutex<HashMap<String, (u32, Instant)>>> = OnceLock::new();
    T.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn is_locked(email: &str) -> bool {
    let t = table().lock().unwrap_or_else(|e| e.into_inner());
    matches!(t.get(email), Some((n, since)) if *n >= MAX_FAILURES && since.elapsed() < WINDOW)
}

pub fn record_failure(email: &str) {
    let mut t = table().lock().unwrap_or_else(|e| e.into_inner());
    if t.len() >= MAX_TRACKED {
        t.retain(|_, (_, since)| since.elapsed() < WINDOW);
    }
    let entry = t.entry(email.to_string()).or_insert((0, Instant::now()));
    if entry.1.elapsed() >= WINDOW {
        *entry = (0, Instant::now());
    }
    entry.0 += 1;
}

pub fn clear(email: &str) {
    table().lock().unwrap_or_else(|e| e.into_inner()).remove(email);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locks_after_ten_failures() {
        let email = "throttle-test@example.org";
        for _ in 0..9 {
            record_failure(email);
        }
        assert!(!is_locked(email));
        record_failure(email);
        assert!(is_locked(email));
        clear(email);
        assert!(!is_locked(email));
    }
}
