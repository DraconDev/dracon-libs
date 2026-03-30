//! Dracon Libraries - Public collection of reusable Rust libraries
//!
//! This is the workspace root for dracon-libs. Individual crates can be used directly:
//!
//! ## Available crates
//!
//! - `dracon-git` - Git operations and utilities
//! - `dracon-terminal-engine` - Terminal UI engine
//! - `dracon-system` - System utilities
//! - `dracon-tts-runtime` - Text-to-speech runtime
//! - `dracon-stt-runtime` - Speech-to-text runtime
//! - `dracon-video-runtime` - Video processing runtime
//! - `dracon-memory-runtime` - Memory/runtime utilities
//!
//! ## Usage
//!
//! ```toml
//! [dependencies]
//! dracon-git = { git = "https://github.com/DraconDev/dracon-libs" }
//! dracon-terminal-engine = { git = "https://github.com/DraconDev/dracon-libs" }
//! ```
//!
//! Or use specific paths in a Cargo workspace.

pub mod git {}
pub mod terminal {}
pub mod system {}
pub mod media {}
pub mod memory {}
