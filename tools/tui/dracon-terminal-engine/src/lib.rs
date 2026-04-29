#![warn(missing_docs)]

//! # Dracon Terminal Engine
//!
//! A z-indexed, event-driven terminal compositor runtime written in Rust.
//!
//! ## Architecture
//!
//! The engine is organized into several layers:
//!
//! - **Core** — [`Terminal`] wraps the terminal in raw mode with RAII cleanup.
//! - **Compositor** — [`Plane`] layers are composited via [`Compositor`] into a
//!   single frame. Supports TrueColor, style flags, opacity, and per-plane
//!   visual filters (Dim, Invert, Scanline, Pulse, Glitch).
//! - **Input** — [`InputReader`] and [`Parser`] decode SGR mouse events and
//!   extended keyboard sequences (chords, modifiers, extra buttons).
//! - **Widgets** — [`Editor`] provides a full-featured code editor with syntax
//!   highlighting via syntect. [`Input`] is a single-line text input widget.
//! - **Integration** — [`ratatui`] bridge lets you drop in any ratatui widget.
//! - **Visuals** — [`icons`] for file-type icons, [`osc`] for clipboard,
//!   hyperlinks, bell, and notifications. [`begin_sync`]/[`end_sync`] implement
//!   terminal mode 2026 for synchronized tear-free output.
//! - **Backend** — [`tty`] wraps low-level POSIX terminal ioctls.
//! - **System** — [`SystemMonitor`] collects CPU, memory, disk, and process
//!   metrics.
//!
//! ## Example
//!
//! ```no_run
//! use dracon_terminal_engine::core::terminal::Terminal;
//! use dracon_terminal_engine::compositor::{Cell, Color, Plane, Styles};
//!
//! let mut terminal = Terminal::new(std::io::stdout()).unwrap();
//! let mut hud = Plane::new(0, 40, 10);
//! hud.set_z_index(50);
//!
//! let cell = Cell {
//!     char: ' ',
//!     fg: Color::Rgb(0, 255, 136),
//!     bg: Color::Rgb(0, 30, 20),
//!     style: Styles::empty(),
//!     transparent: false,
//!     skip: false,
//! };
//! hud.fill(cell);
//! hud.put_str(1, 1, "SYSTEM ONLINE");
//! terminal.write_all(hud.render().as_bytes()).unwrap();
//! ```
//!
//! ## Version
//!
//! v19.2.2

pub mod backend;
pub mod compositor;
pub mod contracts;
pub mod core;
pub mod input;
pub mod integration;
pub mod layout;
pub(crate) mod system;

pub mod utils;
pub mod visuals;
pub mod widgets;

pub use compositor::{Cell, Color, Compositor, Plane, Styles};
pub use core::terminal::Terminal;
pub use input::{InputReader, Parser};

#[doc(hidden)]
pub use contracts;