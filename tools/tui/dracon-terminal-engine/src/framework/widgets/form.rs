//! Form widget for grouping labeled input fields.
//!
//! A vertical layout of labeled form fields with validation support.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct FormField {
    pub label: String,
    pub value: String,
    pub error: Option<String>,
}

pub struct Form {
    id: WidgetId,
    fields: Vec<FormField>,
    focused_field: usize,
    theme: Theme,
}

impl Form {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            fields: Vec::new(),
            focused_field: 0,
            theme: Theme::default(),
        }
    }

    pub fn add_field(mut self, label: &str) -> Self {
        self.fields.push(FormField {
            label: label.to_string(),
            value: String::new(),
            error: None,
        });
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_field_value(&mut self, index: usize, value: &str) {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.value = value.to_string();
        }
    }

    pub fn set_field_error(&mut self, index: usize, error: &str) {
        if let Some(ref mut field) = self.fields.get_mut(index) {
            field.error = Some(error.to_string());
        }
    }
}

impl crate::framework::widget::Widget for Form {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;

        for (i, field) in self.fields.iter().enumerate() {
            if i >= plane.height as usize {
                break;
            }
            let is_focused = i == self.focused_field;
            let label_text = format!("{}: ", field.label);
            let value_text = if field.value.is_empty() {
                "_".to_string()
            } else {
                field.value.clone()
            };

            let fg = if is_focused { self.theme.accent } else { self.theme.fg };
            let err_fg = self.theme.error_fg;

            for (j, c) in label_text.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg: self.theme.bg,
                        style: Styles::BOLD,
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let value_start = label_text.width();
            for (j, c) in value_text.chars().take(width.saturating_sub(value_start)).enumerate() {
                let idx = (i as u16 * plane.width + (value_start + j) as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            if let Some(ref error) = field.error {
                for (j, c) in error.chars().take(width.saturating_sub(value_start)).enumerate() {
                    let idx = (i as u16 * plane.width + (value_start + j) as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: err_fg,
                            bg: self.theme.bg,
                            style: Styles::empty(),
                            transparent: false,
                            skip: false,
                        };
                    }
                }
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::KeyCode;
        match key.code {
            KeyCode::Down => {
                if self.focused_field < self.fields.len().saturating_sub(1) {
                    self.focused_field += 1;
                }
                true
            }
            KeyCode::Up => {
                if self.focused_field > 0 {
                    self.focused_field -= 1;
                }
                true
            }
            _ => false,
        }
    }
}