use std::collections::HashMap;
use std::time::{Duration, Instant};

use rand::Rng;

const TOKEN_TTL: Duration = Duration::from_secs(60);

struct PendingConfirmation {
    plan_hash: String,
    expires_at: Instant,
    consumed: bool,
}

pub struct ConfirmationStore {
    entries: HashMap<String, PendingConfirmation>,
}

impl ConfirmationStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    fn prune(&mut self, now: Instant) {
        self.entries
            .retain(|_, v| v.expires_at > now && !v.consumed);
    }

    pub fn issue(&mut self, plan_hash: String) -> String {
        let now = Instant::now();
        self.prune(now);
        let mut rng = rand::thread_rng();
        let token: String = (0..24)
            .map(|_| format!("{:x}", rng.gen_range(0..16)))
            .collect();
        self.entries.insert(
            token.clone(),
            PendingConfirmation {
                plan_hash,
                expires_at: now + TOKEN_TTL,
                consumed: false,
            },
        );
        token
    }

    /// One-shot consume: token must match `plan_hash`, not expired, not reused.
    pub fn consume(&mut self, token: &str, plan_hash: &str) -> Result<(), String> {
        let now = Instant::now();
        self.prune(now);
        let entry = self
            .entries
            .get_mut(token)
            .ok_or_else(|| "invalid or expired confirmation token".to_string())?;
        if entry.consumed {
            return Err("confirmation token already used".to_string());
        }
        if now > entry.expires_at {
            return Err("confirmation token expired".to_string());
        }
        if entry.plan_hash != plan_hash {
            return Err("confirmation token does not match this action".to_string());
        }
        entry.consumed = true;
        Ok(())
    }
}

impl Default for ConfirmationStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_one_shot_and_hash_binding() {
        let mut s = ConfirmationStore::new();
        let h = "abc123".to_string();
        let t = s.issue(h.clone());
        assert!(s.consume(&t, &h).is_ok());
        assert!(s.consume(&t, &h).is_err());
    }

    #[test]
    fn wrong_hash_rejected() {
        let mut s = ConfirmationStore::new();
        let t = s.issue("plan-a".to_string());
        assert!(s.consume(&t, "plan-b").is_err());
    }
}
