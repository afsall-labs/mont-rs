//! montrs-core: The core runtime and architectural primitives for MontRS.
//! This crate defines the Module trait, AppSpec, and the reactive Signal system.
//! It serves as the backbone for deterministic, trait-driven application initialization.

pub mod signals;
pub mod router;
pub mod env;
pub mod limiter;
pub mod features;

pub use signals::Signal;
pub use router::{Router, Loader, Action, LoaderCtx, ActionCtx};
pub use env::{EnvConfig, EnvConfigExt, TypedEnv, FromEnv, EnvError};
pub use limiter::{Limiter, GovernorLimiter};
pub use features::{FeatureManager, UserContext, FeatureFlag, Segment, Rule};

use std::error::Error;
use async_trait::async_trait;

/// Represents the execution target for the application.
/// Allows for conditional logic based on where the code is running.
pub enum Target {
    Server,
    Wasm,
    Edge,
    Desktop,
    MobileAndroid,
    MobileIos,
}

/// The core trait for modular application components (Modules).
/// Modules are the unit of composition in MontRS, similar to pallets in Substrate.
#[async_trait]
pub trait Module<C: AppConfig>: Send + Sync + 'static {
    /// Unique identifier for the module.
    fn name(&self) -> &'static str;
    
    /// Initialization hook called during application bootstrap.
    /// Provides access to the global configuration and environment.
    async fn init(&self, ctx: &mut ModuleContext<C>) -> Result<(), Box<dyn Error + Send + Sync>>;
    
    /// Hook to register routes (loaders/actions) with the application router.
    fn register_routes(&self, _router: &mut Router<C>) {}
}

/// Context passed to modules during initialization.
pub struct ModuleContext<'a, C: AppConfig> {
    pub config: &'a C,
    pub env: &'a dyn EnvConfig,
}

/// Trait defining the global application requirements.
/// Every MontRS app must provide a custom config and error type.
pub trait AppConfig: Sized + Send + Sync + 'static {
    type Error: Error + Send + Sync;
    type Env: EnvConfig;
}

/// The AppSpec is a deterministic blueprint of the entire application.
/// It contains the configuration, modules, environment, and routing table.
pub struct AppSpec<C: AppConfig> {
    pub config: C,
    pub modules: Vec<Box<dyn Module<C>>>,
    pub env: C::Env,
    pub router: Router<C>,
    pub target: Target,
}

impl<C: AppConfig> AppSpec<C> {
    /// Creates a new, empty AppSpec with the provided config and environment.
    pub fn new(config: C, env: C::Env) -> Self {
        Self {
            config,
            modules: Vec::new(),
            env,
            router: Router::new(),
            target: Target::Server,
        }
    }

    /// Adds a module to the application specification.
    pub fn with_module(mut self, module: Box<dyn Module<C>>) -> Self {
        self.modules.push(module);
        self
    }

    /// Sets the execution target for the application.
    pub fn with_target(mut self, target: Target) -> Self {
        self.target = target;
        self
    }
}
