//! Core widget trait for framework widgets.
//!
//! All framework widgets implement this trait to enable composition,
//! focus management, and event routing.

use crate::compositor::Plane;
use crate::input::event::{KeyEvent, MouseEventKind};
use ratatui::layout::Rect;

/// Unique identifier for a widget (for event routing and state management).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WidgetId(pub usize);

impl Default for WidgetId {
    fn default() -> Self {
        Self(0)
    }
}

impl WidgetId {
    /// Creates a new `WidgetId` with the given numeric value.
    pub fn new(id: usize) -> Self {
        Self(id)
    }
}

/// Trait implemented by all framework widgets.
///
/// Provides a consistent interface for rendering, event handling,
/// and focus management across all widgets.
pub trait Widget {
    /// Returns the unique identifier for this widget.
    fn id(&self) -> WidgetId;

    /// Returns true if this widget can receive focus.
    fn focusable(&self) -> bool {
        true
    }

    /// Renders the widget into a `Plane` at the given area.
    fn render(&self, area: Rect) -> Plane;

    /// Handles a keyboard event.
    /// Returns `true` if the event was consumed, `false` if it should bubble.
    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    /// Handles a mouse event within the widget's bounds.
    /// Returns `true` if the event was consumed.
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }
}