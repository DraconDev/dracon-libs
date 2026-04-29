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
//! - [`ServiceLevel`] — service tier (free, standard, priority)
//!
//! ## Usage
//!
//! ```ignore
//! use dracon_ai_contracts::{RoutingTask, SelectionConstraints};
//! let task = RoutingTask::Code;
//! let constraints = SelectionConstraints { max_latency_ms: 2000, ..Default::default() };
//! ```

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingTask {
    General,
    Code,
    Research,
    Creative,
}

impl RoutingTask {
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

#[derive(Debug, Clone, Default)]
pub struct SelectionConstraints {
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
}
