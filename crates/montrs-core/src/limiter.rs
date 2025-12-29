//! montrs-core/src/limiter.rs: Rate limiting primitives.
//! This module provides a generic Limiter trait and a concrete implementation
//! using the governor crate for sophisticated rate limiting strategies.

use governor::{Quota, RateLimiter, clock::DefaultClock, state::InMemoryState, state::NotKeyed};
use nonzero_ext::nonzero;
use std::num::NonZeroU32;

/// Trait for components that can perform rate limiting checks.
pub trait Limiter: Send + Sync + 'static {
    /// Returns true if the request is allowed, false otherwise.
    fn check(&self) -> bool;
}

/// A rate limiter implementation backed by the governor crate.
/// Uses an in-memory state and a simple per-second quota.
pub struct GovernorLimiter {
    limiter: RateLimiter<NotKeyed, InMemoryState, DefaultClock>,
}

impl GovernorLimiter {
    /// Creates a new GovernorLimiter with the specified allows requests per second.
    pub fn new(per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(per_second).unwrap_or(nonzero!(1u32)));
        Self {
            limiter: RateLimiter::direct(quota),
        }
    }
}

impl Limiter for GovernorLimiter {
    fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }
}
