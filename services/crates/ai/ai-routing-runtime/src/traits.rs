use async_trait::async_trait;

use crate::routing::SelectionConstraints;

#[async_trait]
pub trait AiModelStore: Send + Sync {
    async fn get_best_model(
        &self,
        task: &str,
        constraints: SelectionConstraints,
    ) -> anyhow::Result<(String, bool)>;

    async fn get_leaderboard(
        &self,
        req: ai_service::LeaderboardRequest,
    ) -> anyhow::Result<ai_service::LeaderboardResponse>;

    async fn mark_failure(&self, model_id: &str) -> anyhow::Result<()>;

    async fn update_latency(&self, model_id: &str, latency_ms: u64) -> anyhow::Result<()>;
}
