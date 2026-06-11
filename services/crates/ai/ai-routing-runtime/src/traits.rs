//! AI model leaderboard and model-selection store contracts.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::routing::SelectionConstraints;

/// Request parameters for fetching a model leaderboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    /// Optional task filter.
    pub task: Option<String>,
    /// Optional maximum number of entries.
    pub limit: Option<usize>,
}

/// Model leaderboard response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    /// Ranked model entries.
    pub models: Vec<LeaderboardEntry>,
    /// Highest observed quality score.
    pub max_quality_score: f32,
    /// Highest observed coding score.
    pub max_coding_score: f32,
}

/// Ranked model entry in a leaderboard response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    /// Model identifier.
    pub model_id: String,
    /// Quality score.
    pub quality_score: f32,
    /// Coding score.
    pub coding_score: f32,
    /// Observed latency in milliseconds.
    pub latency_ms: u64,
}

/// Async model-selection and telemetry store.
#[async_trait]
pub trait AiModelStore: Send + Sync {
    /// Return the best model id and whether it is a paid model.
    async fn get_best_model(
        &self,
        task: &str,
        constraints: SelectionConstraints,
    ) -> anyhow::Result<(String, bool)>;

    /// Return leaderboard entries for `req`.
    async fn get_leaderboard(&self, req: LeaderboardRequest)
        -> anyhow::Result<LeaderboardResponse>;

    /// Mark a model as having failed recently.
    async fn mark_failure(&self, model_id: &str) -> anyhow::Result<()>;

    /// Update observed latency for a model.
    async fn update_latency(&self, model_id: &str, latency_ms: u64) -> anyhow::Result<()>;
}
