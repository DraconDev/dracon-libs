//! MiniMax native API.
//!
//! MiniMax uses a custom protocol with its own auth header and message
//! format. A direct MiniMax adapter is planned for a future version of
//! `ai-lib`.
//!
//! For now, use the OpenAI adapter with MiniMax's OpenAI-compatible
//! endpoint if available, or call MiniMax directly via reqwest.

#![allow(dead_code)]
