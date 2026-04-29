#![warn(missing_docs)]

//! Dracon AI Runtime Contracts — ChatMessage, ChatRequest, and AiProvider trait.
//!
//! Defines the runtime-facing contracts for AI providers and chat protocols.
//!
//! ## Modules
//!
//! - [`models`] — `ChatMessage`, `ChatRequest`, `ChatResponse`
//! - [`traits`] — `AiProvider` async trait for AI backend implementations

/// Chat message models for request/response handling.
pub mod models;
/// AI provider trait for runtime implementations.
pub mod traits;
