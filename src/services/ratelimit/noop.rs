//! No-op rate limiter used when Redis/Upstash is not configured.
//!
//! Allows all requests without overhead or state tracking.

use super::{RateLimitDecision, RateLimitError, RateLimiter};
use async_trait::async_trait;

/// A no-op rate limiter that allows all requests unconditionally.
/// Active only when Redis / Upstash is unconfigured.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoOpRateLimiter;

#[async_trait]
impl RateLimiter for NoOpRateLimiter {
    async fn check(&self, _key: &str) -> Result<RateLimitDecision, RateLimitError> {
        Ok(RateLimitDecision {
            allowed: true,
            remaining: u64::MAX,
            limit: 0,
            retry_after: None,
        })
    }

    fn algorithm_name(&self) -> &'static str {
        "disabled"
    }

    fn is_enabled(&self) -> bool {
        false
    }
}
