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

/// Conversation participant role.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Role {
    /// User-authored message.
    User,
    /// Assistant-authored message.
    Assistant,
    /// System or policy-authored message.
    System,
}

/// Stored conversation entry shared by memory consumers and runtimes.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Participant role for the message.
    pub role: Role,
    /// Message text.
    pub content: String,
    /// Optional timestamp string supplied by the caller.
    pub timestamp: Option<String>,
}

/// Extracted user fact with a confidence score.
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFact {
    /// Fact text.
    pub fact: String,
    /// Confidence score in the range expected by callers.
    pub confidence: f32,
}

/// Async storage contract for conversations and semantic recall.
#[async_trait]
pub trait MemoryStore: Send + Sync {
    /// Persist one conversation entry.
    async fn store_conversation(&self, conversation: &Conversation) -> anyhow::Result<()>;
    /// Search for conversations similar to `query`.
    async fn search_similar(&self, query: &str, limit: usize) -> anyhow::Result<Vec<Conversation>>;
    /// Return the most recent conversation entries.
    async fn get_recent(&self, limit: usize) -> anyhow::Result<Vec<Conversation>>;
}

/// Async text embedding contract.
#[async_trait]
pub trait TextEmbedder: Send + Sync {
    /// Embed `text` into a fixed-dimensional vector.
    async fn embed(&self, text: &str) -> anyhow::Result<Vec<f32>>;
    /// Return the embedding dimension.
    fn dimension(&self) -> usize;
}
