//! Search input widget with query text and clear button.
//!
//! A single-line text input optimized for search queries.

use crate::framework::widget::WidgetId;
use crate::framework::widget::Widget;
use ratatui::layout::Rect;

pub struct SearchInput {
    id: WidgetId,
    base: super::text_input_base::BaseInput,
}

impl SearchInput {
    /// Creates a new search input with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            base: super::text_input_base::BaseInput::new(id, "Search..."),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(self, theme: crate::framework::theme::Theme) -> Self {
        Self {
            base: self.base.with_theme(theme),
            ..self
        }
    }

    /// Registers a callback when the user submits the search (Enter key).
    pub fn on_submit(self, f: impl FnMut(&str) + 'static) -> Self {
        Self {
            base: self.base.on_submit(f),
            ..self
        }
    }

    /// Clears the search query.
    pub fn clear(&mut self) {
        self.base.clear();
    }

    /// Returns the current search query.
    pub fn query(&self) -> &str {
        &self.base.text
    }
}

impl Widget for SearchInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.base.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.base.area.set(area);
    }

    fn render(&self, area: Rect) -> crate::compositor::Plane {
        self.base.render_input(area)
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        self.base.cursor_position()
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        self.base.handle_key(key)
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        self.base.handle_mouse(kind, col, row)
    }
}