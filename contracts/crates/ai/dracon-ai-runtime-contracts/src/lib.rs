#![warn(missing_docs)]

//! Dracon AI Runtime Contracts — ChatMessage, ChatRequest, and AiProvider trait.
//!
//! Defines the runtime-facing contracts for AI providers and chat protocols.
//!
//! ## Modules
//!
//! - [`models`] — `ChatMessage`, `ChatRequest`, `ChatResponse`
//! - [`traits`] — `AiProvider` async trait for AI backend implementations

pub mod models;
pub mod traits;
