//! montrs-core/src/features.rs: Dynamic feature flags and user segmentation.
//! This module allows for runtime feature toggling and segment-based
//! targeting to support A/B testing and phased rollouts.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for an individual feature flag.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    pub name: String,
    pub enabled: bool,
    /// Optional whitelist of segments that have access to this feature.
    pub segment_whitelist: Vec<String>,
}

/// Defines a group of users based on specific matching rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub rules: Vec<Rule>,
}

/// Evaluation rules for determining segment membership.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Rule {
    /// Matches a specific user attribute.
    AttributeMatch { key: String, value: String },
    /// Matches a percentage of users deterministically via hashing.
    PercentageMatch(u8),
}

/// The feature manager responsible for evaluating flags and segments.
pub struct FeatureManager {
    flags: HashMap<String, FeatureFlag>,
    segments: HashMap<String, Segment>,
}

impl FeatureManager {
    /// Creates a new, empty FeatureManager.
    pub fn new() -> Self {
        Self {
            flags: HashMap::new(),
            segments: HashMap::new(),
        }
    }

    /// Evaluates if a feature flag is enabled for a given user context.
    pub fn is_enabled(&self, flag_name: &str, user_ctx: &UserContext) -> bool {
        if let Some(flag) = self.flags.get(flag_name) {
            if !flag.enabled {
                return false;
            }
            // If no whitelist, feature is globally enabled.
            if flag.segment_whitelist.is_empty() {
                return true;
            }
            // Check if user is in any whitelisted segment.
            for segment_id in &flag.segment_whitelist {
                if self.is_in_segment(segment_id, user_ctx) {
                    return true;
                }
            }
        }
        false
    }

    /// Internal check to see if a user context satisfies a segment's rules.
    fn is_in_segment(&self, segment_id: &str, user_ctx: &UserContext) -> bool {
        if let Some(segment) = self.segments.get(segment_id) {
            for rule in &segment.rules {
                match rule {
                    Rule::AttributeMatch { key, value } => {
                        if user_ctx.attributes.get(key) != Some(value) {
                            return false;
                        }
                    }
                    Rule::PercentageMatch(pct) => {
                        // Deterministic hash-based percentage match
                        let hash = hash_string(&user_ctx.id);
                        if (hash % 100) as u8 >= *pct {
                            return false;
                        }
                    }
                }
            }
            return true;
        }
        false
    }
}

/// Context representing the current user for feature evaluation.
pub struct UserContext {
    pub id: String,
    pub attributes: HashMap<String, String>,
}

/// Simple deterministic string hashing for percentage rollouts.
fn hash_string(s: &str) -> u32 {
    let mut h = 0u32;
    for b in s.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    h
}
