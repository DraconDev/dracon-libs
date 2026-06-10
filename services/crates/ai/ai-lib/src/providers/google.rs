//! Google Gemini API.
//!
//! Stub: the public API is reserved for v0.2. The Gemini API is similar
//! to OpenAI but with a different auth scheme (`?key=...` query param
//! instead of Authorization header) and a slightly different request shape.
//!
//! For v0.1, use the OpenAI adapter with a Google-compatible proxy, or
//! call Gemini directly via reqwest.

#![allow(dead_code)]
