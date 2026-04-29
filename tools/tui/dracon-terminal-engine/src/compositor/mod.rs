//! The compositor module provides rendering infrastructure for the terminal engine.

pub mod engine;
pub mod filter;
pub mod plane;

/// Re-exports the core [`Compositor`] type from the [`engine`] module.
pub use engine::Compositor;
/// Re-exports types for plane-based rendering: [`Cell`], [`Color`], [`Plane`], and [`Styles`].
pub use plane::{Cell, Color, Plane, Styles};