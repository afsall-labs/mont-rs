//! montrs-core/src/env.rs: Typed environment variable management.
//! This module provides traits and implementations for accessing environment
//! variables in a type-safe and mockable manner.

use std::error::Error;
use std::fmt;

/// Errors that can occur when retrieving or parsing environment variables.
#[derive(Debug)]
pub enum EnvError {
    MissingKey(String),
    InvalidType(String),
}

impl fmt::Display for EnvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EnvError::MissingKey(k) => write!(f, "Missing environment variable: {}", k),
            EnvError::InvalidType(k) => write!(f, "Invalid type for environment variable: {}", k),
        }
    }
}

impl Error for EnvError {}

/// Trait for types that can be initialized from an environment variable string.
pub trait FromEnv: Sized {
    fn from_env(val: String) -> Result<Self, EnvError>;
}

impl FromEnv for String {
    fn from_env(val: String) -> Result<Self, EnvError> {
        Ok(val)
    }
}

/// Core trait for environment configuration providers.
/// Must be dyn-compatible (no generic methods directly).
pub trait EnvConfig: Send + Sync + 'static {
    /// Retrieves a raw string value for the given key.
    fn get_var(&self, key: &str) -> Result<String, EnvError>;
}

/// Extension trait to provide ergonomic typed access to environment variables.
pub trait EnvConfigExt: EnvConfig {
    /// Retrieves and parses an environment variable into the desired type T.
    fn get<T: FromEnv>(&self, key: &str) -> Result<T, EnvError> {
        self.get_var(key).and_then(T::from_env)
    }
}

impl<T: EnvConfig + ?Sized> EnvConfigExt for T {}

/// Default implementation of EnvConfig that reads from the system's environment.
pub struct TypedEnv {
    // Standard implementation using std::env
}

impl EnvConfig for TypedEnv {
    fn get_var(&self, key: &str) -> Result<String, EnvError> {
        std::env::var(key).map_err(|_| EnvError::MissingKey(key.to_string()))
    }
}
