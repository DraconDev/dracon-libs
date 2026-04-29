//! Toggle switch widget.
//!
//! A toggle switch is a two-state on/off control.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct Toggle {
    id: WidgetId,
    state: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
}

impl Toggle {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            state: false,
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

    pub fn toggle(&mut self) {
        self.state = !self.state;
    }

    pub fn is_on(&self) -> bool {
        self.state
    }
}

impl crate::framework::widget::Widget for Toggle {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let width = area.width.max(4) as usize;
        let height = area.height.max(1) as usize;
        let mut plane = Plane::new(width, height);

        let on_text = if self.state { "[*]" } else { "[ ]" };
        let full_text = format!("{} {}", on_text, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let bg = if self.state {
            self.theme.success_fg
        } else {
            self.theme.inactive_fg
        };

        for (i, c) in full_text.chars().take(width).enumerate() {
            plane.set_cell(
                (start_x + i) as i32,
                start_y as i32,
                Cell::new(c, Styles::default().with_bg(bg).with_fg(self.theme.fg)),
            );
        }

        plane
    }

    fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::KeyCode;
        match key.code {
            KeyCode::Enter | KeyCode::Space => {
                self.toggle();
                if let Some(ref mut cb) = self.on_change {
                    cb(self.state);
                }
                true
            }
            _ => false,
        }
    }
}