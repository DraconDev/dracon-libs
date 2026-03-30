pub mod db;
pub mod embedder;
pub mod memory_contracts;

pub use crate::memory_contracts::{Conversation, MemoryStore, Role, TextEmbedder, UserFact};
pub use db::MemoryDb;
pub use embedder::OnnxEmbedder;

use std::sync::Arc;
use tokio::sync::Mutex;

pub struct MemorySystem {
    db: Arc<Mutex<MemoryDb>>,
    embedder: Arc<Mutex<OnnxEmbedder>>,
}

impl MemorySystem {
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let db = MemoryDb::new(db_path)?;
        let embedder = OnnxEmbedder::new()?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            embedder: Arc::new(Mutex::new(embedder)),
        })
    }

    pub async fn store_conversation(&self, role: Role, content: &str) -> anyhow::Result<i64> {
        let embedding = self.embedder.lock().await.embed(content);
        let db = self.db.lock().await;
        db.store_conversation(role, content, &embedding)
    }

    pub async fn recall_relevant(
        &self,
        query: &str,
        k: usize,
    ) -> anyhow::Result<Vec<Conversation>> {
        let embedding = self.embedder.lock().await.embed(query);
        let db = self.db.lock().await;
        db.search_similar(&embedding, k)
    }

    pub async fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>> {
        let db = self.db.lock().await;
        db.get_recent(limit)
    }

    pub async fn clear(&self) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        db.clear()
    }

    pub async fn store_fact(
        &self,
        category: &str,
        key: &str,
        value: &str,
        source: Option<&str>,
    ) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        db.store_fact(category, key, value, source)
    }

    pub async fn get_fact(&self, category: &str, key: &str) -> anyhow::Result<Option<UserFact>> {
        let db = self.db.lock().await;
        db.get_fact(category, key)
    }

    pub async fn get_facts_by_category(&self, category: &str) -> anyhow::Result<Vec<UserFact>> {
        let db = self.db.lock().await;
        db.get_facts_by_category(category)
    }

    pub async fn get_all_facts(&self) -> anyhow::Result<Vec<UserFact>> {
        let db = self.db.lock().await;
        db.get_all_facts()
    }

    pub async fn get_user_summary(&self) -> anyhow::Result<String> {
        let db = self.db.lock().await;
        db.get_all_facts_summary()
    }

    pub async fn store_facts(&self, facts: &[(String, String, String)]) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        for (category, key, value) in facts {
            db.store_fact(category, key, value, Some("extracted"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_system_new() {
        let memory = MemorySystem::new(":memory:");
        assert!(memory.is_ok());
    }

    #[tokio::test]
    async fn test_memory_system_store_and_recall() {
        let memory = MemorySystem::new(":memory:").unwrap();

        memory
            .store_conversation(Role::User, "I like pizza")
            .await
            .unwrap();
        memory
            .store_conversation(Role::Assistant, "Great choice!")
            .await
            .unwrap();

        let recent = memory.get_recent(10).await.unwrap();
        assert_eq!(recent.len(), 2);
    }

    #[tokio::test]
    async fn test_memory_system_semantic_search() {
        let memory = MemorySystem::new(":memory:").unwrap();

        memory
            .store_conversation(Role::User, "My favorite food is pizza")
            .await
            .unwrap();
        memory
            .store_conversation(Role::User, "The weather is nice today")
            .await
            .unwrap();

        let results = memory
            .recall_relevant("What do I like to eat?", 1)
            .await
            .unwrap();
        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_memory_system_facts() {
        let memory = MemorySystem::new(":memory:").unwrap();

        memory
            .store_fact("personal", "name", "Alice", None)
            .await
            .unwrap();

        let fact = memory.get_fact("personal", "name").await.unwrap();
        assert!(fact.is_some());
        assert_eq!(fact.unwrap().value, "Alice");
    }

    #[tokio::test]
    async fn test_memory_system_user_summary() {
        let memory = MemorySystem::new(":memory:").unwrap();

        memory
            .store_fact("personal", "name", "Alice", None)
            .await
            .unwrap();
        memory
            .store_fact("preferences", "color", "blue", None)
            .await
            .unwrap();

        let summary = memory.get_user_summary().await.unwrap();
        assert!(summary.contains("Alice"));
        assert!(summary.contains("blue"));
    }

    #[tokio::test]
    async fn test_memory_system_clear() {
        let memory = MemorySystem::new(":memory:").unwrap();

        memory.store_conversation(Role::User, "Test").await.unwrap();
        assert_eq!(memory.get_recent(10).await.unwrap().len(), 1);

        memory.clear().await.unwrap();
        assert_eq!(memory.get_recent(10).await.unwrap().len(), 0);
    }
}
