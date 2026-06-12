//! AI routing primitives for service-level selection.

use serde::{Deserialize, Serialize};

pub use dracon_ai_contracts::SelectionConstraints;

/// Service tier used to select model routing behavior.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceLevel {
    /// Free-tier routing.
    #[default]
    Free,
    /// Paid-tier routing.
    Paid,
    /// Enterprise-tier routing.
    Enterprise,
}
