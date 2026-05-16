use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::routing::SelectionConstraints;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardRequest {
    pub task: Option<String>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardResponse {
    pub models: Vec<LeaderboardEntry>,
    pub max_quality_score: f32,
    pub max_coding_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub model_id: String,
    pub quality_score: f32,
    pub coding_score: f32,
    pub latency_ms: u64,
}

#[async_trait]
pub trait AiModelStore: Send + Sync {
    async fn get_best_model(
        &self,
        task: &str,
        constraints: SelectionConstraints,
    ) -> anyhow::Result<(String, bool)>;

    async fn get_leaderboard(&self, req: LeaderboardRequest)
        -> anyhow::Result<LeaderboardResponse>;

    async fn mark_failure(&self, model_id: &str) -> anyhow::Result<()>;

    async fn update_latency(&self, model_id: &str, latency_ms: u64) -> anyhow::Result<()>;
}
