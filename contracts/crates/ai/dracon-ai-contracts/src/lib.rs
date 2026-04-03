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
