#![warn(missing_docs)]

//! Dracon AI Contracts — routing task definitions and selection constraints.
//!
//! Defines how routing tasks are categorized and how model selection
//! constraints are expressed.
//!
//! ## Types
//!
//! - [`RoutingTask`] — task category (General, Code, Research, Creative)
//! - [`SelectionConstraints`] — latency/cost/provider constraints for routing
//!
//! ## Usage
//!
//! ```ignore
//! use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
//! let task = RoutingTask::Code;
//! let constraints = SelectionConstraints { max_tokens: Some(1000), ..Default::default() };
//! ```
//!
//! ## Feature Flags
//!
//! - `contracts` — core routing contracts (default)

use serde::{Deserialize, Serialize};

/// Task category for AI routing decisions.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingTask {
    /// General-purpose tasks
    General,
    /// Code generation, review, and refactoring tasks
    Code,
    /// Research, analysis, and information retrieval tasks
    Research,
    /// Creative writing and brainstorming tasks
    Creative,
}

impl RoutingTask {
    /// Parse a `RoutingTask` from a string slice.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "general" => Some(RoutingTask::General),
            "code" => Some(RoutingTask::Code),
            "research" => Some(RoutingTask::Research),
            "creative" => Some(RoutingTask::Creative),
            _ => None,
        }
    }
}

/// Constraints for model selection during routing.
#[non_exhaustive]
#[derive(Debug, Clone, Default)]
pub struct SelectionConstraints {
    /// Maximum number of tokens in the response.
    pub max_tokens: Option<usize>,
    /// Sampling temperature for generation.
    pub temperature: Option<f32>,
}
