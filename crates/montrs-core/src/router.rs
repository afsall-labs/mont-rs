//! montrs-core/src/router.rs: Explicit routing primitives inspired by Remix.
//! This module defines Loaders for data fetching and Actions for data mutation,
//! ensuring clear boundaries between reads and writes.

use std::collections::HashMap;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::AppConfig;

/// Trait for data loading components. Loaders are responsible for fetching data
/// for a specific route. They are read-only and idempotent.
#[async_trait]
pub trait Loader<C: AppConfig>: Send + Sync + 'static {
    async fn call(&self, ctx: LoaderCtx<C>) -> Result<LoaderResponse, crate::signals::Signal<String>>;
}

/// Trait for data mutation components. Actions are responsible for handling
/// state-changing operations (form submissions, API mutations).
#[async_trait]
pub trait Action<C: AppConfig>: Send + Sync + 'static {
    async fn call(&self, input: serde_json::Value, ctx: ActionCtx<C>) -> Result<ActionResponse, crate::signals::Signal<String>>;
}

/// Context passed to a Loader, providing access to the application configuration.
pub struct LoaderCtx<C: AppConfig> {
    pub config: C,
}

/// Context passed to an Action, providing access to the application configuration.
pub struct ActionCtx<C: AppConfig> {
    pub config: C,
}

/// Standard response format for a Loader.
#[derive(Serialize, Deserialize)]
pub struct LoaderResponse {
    pub data: serde_json::Value,
}

/// Standard response format for an Action.
#[derive(Serialize, Deserialize)]
pub struct ActionResponse {
    pub data: serde_json::Value,
}

/// The Application Router which maintains a mapping of paths to Loaders and Actions.
pub struct Router<C: AppConfig> {
    loaders: HashMap<String, Box<dyn Loader<C>>>,
    actions: HashMap<String, Box<dyn Action<C>>>,
}

impl<C: AppConfig> Router<C> {
    /// Initializes a new, empty router.
    pub fn new() -> Self {
        Self {
            loaders: HashMap::new(),
            actions: HashMap::new(),
        }
    }

    /// Registers a loader for a specific path.
    pub fn register_loader(&mut self, path: &str, loader: Box<dyn Loader<C>>) {
        self.loaders.insert(path.to_string(), loader);
    }

    /// Registers an action for a specific path.
    pub fn register_action(&mut self, path: &str, action: Box<dyn Action<C>>) {
        self.actions.insert(path.to_string(), action);
    }
}
