#![warn(missing_docs)]

//! Dracon Memory Contracts — trait contracts for the memory runtime.
//!
//! Defines the public-facing traits for the memory system. The runtime
//! (`dracon-memory-runtime`) implements these; consumers should depend on
//! this crate to get the traits without pulling in heavy ONNX/SQLite deps.
//!
//! ## Traits
//!
//! - [`MemoryStore`] — async conversation storage and recall
//! - [`TextEmbedder`] — async text embedding
//!
//! ## Types
//!
//! - [`Role`] — User / Assistant / System conversation roles
//! - [`Conversation`] — stored conversation entry
//! - [`UserFact`] — key-value fact with confidence score

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    User,
    Assistant,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub role: Role,
    pub content: String,
    pub timestamp: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFact {
    pub fact: String,
    pub confidence: f32,
}

#[async_trait]
pub trait MemoryStore: Send + Sync {
    async fn store_conversation(&self, conversation: &Conversation) -> anyhow::Result<()>;
    async fn search_similar(&self, query: &str, limit: usize) -> anyhow::Result<Vec<Conversation>>;
    async fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>>;
}

#[async_trait]
pub trait TextEmbedder: Send + Sync {
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    fn dimension(&self) -> usize;
}
