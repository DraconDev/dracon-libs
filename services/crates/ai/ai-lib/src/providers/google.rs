//! Google Gemini API.
//!
//! The Gemini API is similar to OpenAI but with a different auth scheme
//! (`?key=...` query param instead of Authorization header) and a slightly
//! different request shape. A direct Gemini adapter is planned for a future
//! version of `ai-lib`.
//!
//! For now, use the OpenAI adapter with a Google-compatible proxy, or
//! call Gemini directly via reqwest.

#![allow(dead_code)]
