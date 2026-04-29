//! Search input widget with query text and clear button.
//!
//! A single-line text input optimized for search queries.

use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct SearchInput {
    id: WidgetId,
    query: String,
    cursor_pos: usize,
    theme: Theme,
    on_submit: Option<Box<dyn FnMut(&str)>>,
}

impl SearchInput {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            query: String::new(),
            cursor_pos: 0,
            theme: Theme::default(),
            on_submit: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_submit(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn clear(&mut self) {
        self.query.clear();
        self.cursor_pos = 0;
    }

    pub fn query(&self) -> &str {
        &self.query
    }
}

impl crate::framework::widget::Widget for SearchInput {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

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

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::KeyCode;
        match key.code {
            KeyCode::Enter => {
                if let Some(ref mut cb) = self.on_submit {
                    cb(&self.query);
                }
                true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 && !self.query.is_empty() {
                    let char_width = self.query.chars().nth(self.cursor_pos - 1).map(|c| c.width().unwrap_or(1)).unwrap_or(1);
                    self.query.truncate(self.query.len().saturating_sub(char_width.max(1)));
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        if col < self.query.len() as u16 {
            self.cursor_pos = col as usize;
            true
        } else {
            false
        }
    }
}