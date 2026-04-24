use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Simple per-key sliding window limiter (default: N events per minute).
pub struct SlidingWindowLimiter {
    window: Duration,
    max_events: usize,
    events: Mutex<VecDeque<Instant>>,
}

pub struct DailyQuotaLimiter {
    quotas: HashMap<String, usize>,
    counters: Mutex<HashMap<String, (chrono::NaiveDate, usize)>>,
}

impl SlidingWindowLimiter {
    pub fn per_minute(max_events: usize) -> Self {
        Self {
            window: Duration::from_secs(60),
            max_events,
            events: Mutex::new(VecDeque::new()),
        }
    }

    /// Returns `Ok(())` or suggested retry-after in milliseconds.
    pub fn check(&self) -> Result<(), u64> {
        let now = Instant::now();
        let mut q = self.events.lock().map_err(|_| 1_000u64)?;
        while let Some(front) = q.front() {
            if now.duration_since(*front) > self.window {
                q.pop_front();
            } else {
                break;
            }
        }
        if q.len() >= self.max_events {
            let front = *q.front().unwrap();
            let retry_ms = self
                .window
                .saturating_sub(now.duration_since(front))
                .as_millis() as u64;
            return Err(retry_ms.max(1));
        }
        q.push_back(now);
        Ok(())
    }
}

impl DailyQuotaLimiter {
    pub fn with_defaults() -> Self {
        let mut quotas = HashMap::new();
        quotas.insert("groq".into(), 14_400);
        quotas.insert("gemini".into(), 1_500);
        quotas.insert("openrouter".into(), 5_000);
        Self {
            quotas,
            counters: Mutex::new(HashMap::new()),
        }
    }

    pub fn check_and_increment(&self, provider: &str) -> Result<(), String> {
        let today = chrono::Local::now().date_naive();
        let mut counters = self
            .counters
            .lock()
            .map_err(|_| "daily quota lock poisoned".to_string())?;
        let entry = counters.entry(provider.to_string()).or_insert((today, 0));
        if entry.0 != today {
            *entry = (today, 0);
        }
        let Some(limit) = self.quotas.get(provider) else {
            return Ok(());
        };
        if entry.1 >= *limit {
            return Err(format!("Daily quota exhausted for {provider}"));
        }
        entry.1 += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allows_under_cap() {
        let l = SlidingWindowLimiter::per_minute(10);
        for _ in 0..10 {
            assert!(l.check().is_ok());
        }
    }

    #[test]
    fn rejects_over_cap() {
        let l = SlidingWindowLimiter::per_minute(3);
        assert!(l.check().is_ok());
        assert!(l.check().is_ok());
        assert!(l.check().is_ok());
        assert!(l.check().is_err());
    }

    #[test]
    fn daily_quota_rejects_after_limit() {
        let q = DailyQuotaLimiter::with_defaults();
        for _ in 0..1500 {
            q.check_and_increment("gemini").unwrap();
        }
        assert!(q.check_and_increment("gemini").is_err());
    }
}
