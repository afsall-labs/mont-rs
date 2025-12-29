//! montrs-test: Utilities for deterministic testing of MontRS applications.
//! This crate provides a mockable environment and a test runtime for
//! verifying application logic in-process.

use montrs_core::env::EnvError;
use montrs_core::{AppConfig, AppSpec, EnvConfig};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// A mock environment configuration provider for testing.
/// Allows setting variables manually to simulate different environments.
pub struct TestEnv {
    vars: Arc<RwLock<HashMap<String, String>>>,
}

impl TestEnv {
    /// Creates a new, empty TestEnv.
    pub fn new() -> Self {
        Self {
            vars: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Sets an environment variable for the test session.
    pub fn set(&self, key: &str, value: &str) {
        let mut vars = self.vars.write().unwrap();
        vars.insert(key.to_string(), value.to_string());
    }
}

impl EnvConfig for TestEnv {
    fn get_var(&self, key: &str) -> Result<String, EnvError> {
        let vars = self.vars.read().unwrap();
        vars.get(key)
            .cloned()
            .ok_or_else(|| EnvError::MissingKey(key.to_string()))
    }
}

/// A specialized runtime for executing MontRS components in a test context.
pub struct TestRuntime<C: AppConfig> {
    pub spec: AppSpec<C>,
}

impl<C: AppConfig> TestRuntime<C> {
    /// Creates a new TestRuntime with the provided AppSpec.
    pub fn new(spec: AppSpec<C>) -> Self {
        Self { spec }
    }

    /// Executes a closure within the test runtime context.
    pub async fn execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&AppSpec<C>) -> R,
    {
        // In a real implementation, this would set up the reactive context,
        // potentially a tokio task local for the runtime, etc.
        f(&self.spec)
    }
}
