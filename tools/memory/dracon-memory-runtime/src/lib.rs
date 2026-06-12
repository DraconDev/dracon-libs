#![warn(missing_docs)]

//! Dracon Memory Runtime — SQLite + ONNX embeddings for conversation storage and semantic search.
//!
//! ## Crates
//!
//! - [`MemorySystem`] — main struct combining DB + embedder
//! - [`MemoryDb`] — SQLite persistence (in-memory or file-backed)
//! - [`OnnxEmbedder`] — ONNX-based text embedding via BGE model
//! - [`MemoryStore`] trait — storage contract
//! - [`TextEmbedder`] trait — embedding contract
//!
//! ## Environment Variables
//!
//! - `DRACON_MODEL_PATH` — path to ONNX model (default: `assets/bge-small-en-v1.5.onnx`)
//! - `DRACON_TOKENIZER_PATH` — path to tokenizer (default: `assets/tokenizer.json`)
//!
//! ## Example
//!
//! ```ignore
//! use dracon_memory_runtime::MemorySystem;
//! let mem = MemorySystem::new(":memory:")?;
//! mem.store_conversation(Role::User, "Hello").await?;
//! ```

/// SQLite-backed memory store with ONNX embedding support.
pub mod db;
/// ONNX and fallback embedding backend.
pub mod embedder;
/// Runtime-side memory contract types.
pub mod memory_contracts;

pub use crate::memory_contracts::{Conversation, MemoryStore, Role, TextEmbedder, UserFact};
pub use db::MemoryDb;
pub use embedder::OnnxEmbedder;

use std::sync::Arc;
use tokio::sync::Mutex;

/// High-level memory system combining the SQLite database and embedder.
pub struct MemorySystem {
    db: Arc<Mutex<MemoryDb>>,
    embedder: Arc<Mutex<OnnxEmbedder>>,
}

impl MemorySystem {
    /// Create a memory system using `db_path` for SQLite storage.
    pub fn new(db_path: &str) -> anyhow::Result<Self> {
        let db = MemoryDb::new(db_path)?;
        let embedder = OnnxEmbedder::new()?;

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            embedder: Arc::new(Mutex::new(embedder)),
        })
    }

    /// Store a conversation and return its database row id.
    pub async fn store_conversation(&self, role: Role, content: &str) -> anyhow::Result<i64> {
        let embedding = self.embedder.lock().await.embed(content);
        let db = self.db.lock().await;
        db.store_conversation(role, content, &embedding)
    }

    /// Return conversations most similar to `query`.
    pub async fn recall_relevant(
        &self,
        query: &str,
        k: usize,
    ) -> anyhow::Result<Vec<Conversation>> {
        let embedding = self.embedder.lock().await.embed(query);
        let db = self.db.lock().await;
        db.search_similar(&embedding, k)
    }

    /// Return the most recent conversations.
    pub async fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>> {
        let db = self.db.lock().await;
        db.get_recent(limit)
    }

    /// Delete stored conversations and vector rows.
    pub async fn clear(&self) -> anyhow::Result<()> {
        let db = self.db.lock().await;
        db.clear()
    }

    /// Delete one conversation and its vector row by database id.
    pub async fn delete_conversation(&self, id: i64) -> anyhow::Result<usize> {
        let db = self.db.lock().await;
        db.delete_conversation(id)
    }

    /// Delete conversations created before `timestamp` and their vector rows.
    pub async fn delete_conversations_before(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> anyhow::Result<usize> {
        let db = self.db.lock().await;
        db.delete_conversations_before(timestamp)
    }

    /// Store or update a user fact.
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

    /// Delete one user fact by category and key.
    pub async fn delete_fact(&self, category: &str, key: &str) -> anyhow::Result<usize> {
        let db = self.db.lock().await;
        db.delete_fact(category, key)
    }

    /// Delete all user facts in a category.
    pub async fn delete_facts_by_category(&self, category: &str) -> anyhow::Result<usize> {
        let db = self.db.lock().await;
        db.delete_facts_by_category(category)
    }

    /// Return a user fact by category and key.
    pub async fn get_fact(&self, category: &str, key: &str) -> anyhow::Result<Option<UserFact>> {
        let db = self.db.lock().await;
        db.get_fact(category, key)
    }

    /// Return all user facts in a category.
    pub async fn get_facts_by_category(&self, category: &str) -> anyhow::Result<Vec<UserFact>> {
        let db = self.db.lock().await;
        db.get_facts_by_category(category)
    }

    /// Return all stored user facts.
    pub async fn get_all_facts(&self) -> anyhow::Result<Vec<UserFact>> {
        let db = self.db.lock().await;
        db.get_all_facts()
    }

    /// Return a text summary of all stored user facts.
    pub async fn get_user_summary(&self) -> anyhow::Result<String> {
        let db = self.db.lock().await;
        db.get_all_facts_summary()
    }

    /// Store multiple facts with an extracted source marker.
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
    async fn test_memory_system_deletes_individual_records() {
        let memory = MemorySystem::new(":memory:").unwrap();

        let id = memory
            .store_conversation(Role::User, "delete me")
            .await
            .unwrap();
        memory
            .store_conversation(Role::User, "keep me")
            .await
            .unwrap();
        memory
            .store_fact("personal", "name", "Alice", None)
            .await
            .unwrap();
        memory
            .store_fact("personal", "theme", "dark", None)
            .await
            .unwrap();

        assert_eq!(memory.delete_conversation(id).await.unwrap(), 1);
        assert_eq!(memory.delete_fact("personal", "name").await.unwrap(), 1);

        let recent = memory.get_recent(10).await.unwrap();
        assert_eq!(recent.len(), 1);
        assert_eq!(recent[0].content, "keep me");
        assert!(memory.get_fact("personal", "name").await.unwrap().is_none());
        assert_eq!(
            memory
                .get_fact("personal", "theme")
                .await
                .unwrap()
                .unwrap()
                .value,
            "dark"
        );
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
