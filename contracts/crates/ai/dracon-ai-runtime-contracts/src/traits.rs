use super::models::ChatRequest;
use async_trait::async_trait;

/// Async trait for AI backend providers.
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// Send a chat request and collect the response.
    async fn ask_and_collect(
        &self,
        request: ChatRequest,
    ) -> anyhow::Result<(String, Option<String>)>;
}
