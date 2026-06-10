//! models.dev-backed provider catalog and BYOK config types.
//!
//! This crate is intentionally a **pure data crate**:
//! - no HTTP client
//! - no provider adapters
//! - no env-var auto-read
//! - no file I/O except the explicit `load_config()` helper
//!
//! It gives consumers a shared schema for provider metadata and config
//! files, so `ai-lib`, `dracon-code`, and other tools can all speak the
//! same language without depending on the Dracon AI API.

pub mod catalog;
pub mod config;
pub mod env;

pub use catalog::{Catalog, Cost, Limit, Modalities, Model, Provider};
pub use config::{
    load_config, parse_config, AiModelsConfig, ModelOverride, ProviderConfig,
};
pub use env::{builtin_env_vars, env_var_names_for, BuiltinProvider};
