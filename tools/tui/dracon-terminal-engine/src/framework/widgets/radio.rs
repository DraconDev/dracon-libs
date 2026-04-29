//! Radio button widget.
//!
//! A radio button for mutually exclusive selection within a group.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct Radio {
    id: WidgetId,
    selected: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
}

impl Radio {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            selected: false,
            label: label.to_string(),
            theme: Theme::default(),
            on_change: None,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_change(mut self, f: impl FnMut(bool) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn select(&mut self) {
        self.selected = true;
    }

    pub fn deselect(&mut self) {
        self.selected = false;
    }

    pub fn is_selected(&self) -> bool {
        self.selected
    }
}

impl crate::framework::widget::Widget for Radio {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

        let radio_str = if self.selected { "(o)" } else { "( )" };
        let full_text = format!("{} {}", radio_str, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let fg = if self.selected {
            self.theme.primary_fg
        } else {
            self.theme.fg
        };

        for (i, c) in full_text.chars().take(width).enumerate() {
            let idx = (start_y as u16 * plane.width + (start_x as u16 + i as u16)) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx] = Cell::new(c, Styles::default().with_fg(fg).with_bg(self.theme.bg));
            }
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::KeyCode;
        match key.code {
            KeyCode::Enter | KeyCode::Space => {
                if !self.selected {
                    self.selected = true;
                    if let Some(ref mut cb) = self.on_change {
                        cb(true);
                    }
                }
                true
            }
            _ => false,
        }
    }
}