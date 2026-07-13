// TRRUSTT — Rate Limiter. Token bucket per provider.
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use shared::Result;

struct TokenBucket { tokens: f64, max_tokens: f64, refill_rate: f64, last_refill: Instant }
impl TokenBucket {
    fn new(max: f64, rate: f64) -> Self { Self { tokens: max, max_tokens: max, refill_rate: rate, last_refill: Instant::now() } }
    fn try_consume(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
        if self.tokens >= 1.0 { self.tokens -= 1.0; true } else { false }
    }
}

pub struct RateLimiter { buckets: Arc<Mutex<HashMap<String, TokenBucket>>> }

impl RateLimiter {
    pub fn new() -> Self { Self { buckets: Arc::new(Mutex::new(HashMap::new())) } }

    pub async fn check(&self, provider: &str) -> Result<()> {
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(provider.to_string()).or_insert_with(|| TokenBucket::new(10.0, 1.0));
        if !bucket.try_consume() {
            return Err(shared::AppError::AiRateLimit { provider: provider.to_string(), message: "Rate limit exceeded".into() });
        }
        Ok(())
    }

    pub async fn configure(&self, provider: &str, max: f64, rate: f64) {
        self.buckets.lock().await.insert(provider.to_string(), TokenBucket::new(max, rate));
    }

    pub async fn tokens_remaining(&self, provider: &str) -> f64 {
        self.buckets.lock().await.get(provider).map(|b| b.tokens).unwrap_or(0.0)
    }
}

impl Default for RateLimiter { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_token_bucket() {
        let mut b = TokenBucket::new(10.0, 1.0);
        for _ in 0..10 { assert!(b.try_consume()); }
        assert!(!b.try_consume());
    }
}
