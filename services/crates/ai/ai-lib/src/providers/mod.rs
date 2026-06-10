//! Provider adapters.
//!
//! Each module implements one provider family. Today:
//! - [`openai`] — OpenAI Chat Completions / Image Generation API.
//!   This is also the protocol used by OpenRouter, NVIDIA NIM, Mistral,
//!   DeepSeek, and Apertis — just point `base_url` at the right host.
//! - [`anthropic`] — Anthropic Messages API.
//! - [`google`] — Google Gemini API.
//! - [`minimax`] — MiniMax native API.

pub mod anthropic;
pub mod google;
pub mod minimax;
pub mod openai;
