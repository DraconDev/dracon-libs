use async_trait::async_trait;
use super::models::ChatRequest;

#[async_trait]
pub trait AiProvider: Send + Sync {
    async fn ask_and_collect(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)>;
}
