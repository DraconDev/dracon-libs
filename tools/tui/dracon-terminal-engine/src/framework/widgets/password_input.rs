//! Password input widget with character masking.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A password input widget that masks characters as they're typed.
pub struct PasswordInput {
    id: WidgetId,
    password: String,
    cursor_pos: usize,
    theme: Theme,
    mask_char: char,
    on_submit: Option<Box<dyn FnMut(&str)>>,
    area: std::cell::Cell<Rect>,
}

impl PasswordInput {
    /// Creates a new password input with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            password: String::new(),
            cursor_pos: 0,
            theme: Theme::default(),
            mask_char: '*',
            on_submit: None,
            area: std::cell::Cell::new(Rect::new(0, 0, 30, 1)),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the mask character (default is '*').
    pub fn with_mask_char(mut self, ch: char) -> Self {
        self.mask_char = ch;
        self
    }

    /// Registers a callback when the user submits the password (Enter key).
    pub fn on_submit(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    /// Clears the password.
    pub fn clear(&mut self) {
        self.password.clear();
        self.cursor_pos = 0;
    }

    /// Returns the current password (unmasked).
    pub fn password(&self) -> &str {
        &self.password
    }
}

impl crate::framework::widget::Widget for PasswordInput {
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

        let display = if self.password.is_empty() {
            "Password...".to_string()
        } else {
            self.password.chars().map(|_| self.mask_char).collect::<String>()
        };

        for (i, c) in display.chars().take(width.saturating_sub(2)).enumerate() {
            let idx = (0u16 * plane.width + i as u16) as usize;
            if idx < plane.cells.len() {
                let is_cursor = i == self.cursor_pos && !self.password.is_empty();
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
                    cb(&self.password);
                }
                true
            }
            KeyCode::Backspace => {
                if self.cursor_pos > 0 && !self.password.is_empty() {
                    self.password.pop();
                    self.cursor_pos = self.cursor_pos.saturating_sub(1);
                }
                true
            }
            KeyCode::Char(ch) => {
                self.password.push(ch);
                if self.cursor_pos < self.password.len() {
                    self.cursor_pos = self.password.len();
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
                if self.cursor_pos < self.password.len() {
                    self.cursor_pos += 1;
                }
                true
            }
            KeyCode::Delete => {
                if self.cursor_pos < self.password.len() {
                    self.password.remove(self.cursor_pos);
                }
                true
            }
            KeyCode::Home => {
                self.cursor_pos = 0;
                true
            }
            KeyCode::End => {
                self.cursor_pos = self.password.len();
                true
            }
            _ => false,
        }
    }

    fn handle_mouse(&mut self, _kind: crate::input::event::MouseEventKind, col: u16, _row: u16) -> bool {
        if col < self.password.len() as u16 {
            self.cursor_pos = col as usize;
            true
        } else {
            false
        }
    }
}