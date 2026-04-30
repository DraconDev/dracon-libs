//! Search input widget with query text and clear button.
//!
//! A single-line text input optimized for search queries.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A search input widget with a query buffer and submit callback.
pub struct SearchInput {
    id: WidgetId,
    query: String,
    cursor_pos: usize,
    theme: Theme,
    on_submit: Option<Box<dyn FnMut(&str)>>,
    area: std::cell::Cell<Rect>,
}

impl SearchInput {
    /// Creates a new search input with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            query: String::new(),
            cursor_pos: 0,
            theme: Theme::default(),
            on_submit: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 1)),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Registers a callback when the user submits the search (Enter key).
    pub fn on_submit(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Clears the search query.
    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
    }

    /// Returns the current search query.
    pub fn query(&self) -> &str {
        &self.query
    }
}

impl crate::framework::widget::Widget for SearchInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;

        let display = if self.query.is_empty() {
            "Search...".to_string()
        } else {
            self.query.clone()
        };

        for (i, c) in display.chars().take(width.saturating_sub(2)).enumerate() {
            let idx = (0u16 * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                let is_cursor = i == self.cursor_pos && !self.query.is_empty();
                plane.cells[idx] = Cell {
                    char: c,
                    fg: if is_cursor { self.theme.bg } else { self.theme.fg },
                    bg: if is_cursor { self.theme.fg } else { self.theme.input_bg },
                    style: Styles::empty(),
                    transparent: false,
                    skip: false,
                };
            }
        }

        plane
    }

    fn cursor_position(&self) -> Option<(u16, u16)> {
        let area = self.area.get();
        Some((area.x + self.cursor_pos as u16, area.y))
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Enter => {
                if let Some(ref mut cb) = self.on_submit {
                    cb(&self.query);
                }
                true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 && !self.query.is_empty() {
                    self.query.pop();
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
                true
            }
            KeyCode::Char(ch) => {
                self.query.push(ch);
                if self.cursor_pos < self.query.len() {
                    self.cursor_pos = self.query.len();
                }
                true
            }
            KeyCode::Left => {
                if self.cursor_pos > 0 {
                    self.cursor_pos -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.cursor_pos < self.query.len() {
                    self.cursor_pos += 1;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.query.len() {
                    self.query.remove(self.cursor_pos);
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.query.len();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: crate::input::event::MouseEventKind, col: u16, _row: u16) -> bool {
        if col < self.query.len() as u16 {
            self.cursor_pos = col as usize;
            true
        } else {
            false
        }
    }
}