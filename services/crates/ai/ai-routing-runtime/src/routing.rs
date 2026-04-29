use serde::{Deserialize, Serialize};

pub use dracon_ai_contracts::SelectionConstraints;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceLevel {
    Free,
    Paid,
    Enterprise,
}

impl Default for ServiceLevel {
    fn default() -> Self {
        Self::Free
    }
}
