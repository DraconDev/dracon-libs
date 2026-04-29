//! Checkbox widget.
//!
//! A checkbox is a two-state on/off control with a check mark.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct Checkbox {
    id: WidgetId,
    checked: bool,
    label: String,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(bool)>>,
}

impl Checkbox {
    pub fn new(id: WidgetId, label: &str) -> Self {
        Self {
            id,
            checked: false,
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

    pub fn check(&mut self) {
        self.checked = true;
    }

    pub fn uncheck(&mut self) {
        self.checked = false;
    }

    pub fn toggle(&mut self) {
        self.checked = !self.checked;
    }

    pub fn is_checked(&self) -> bool {
        self.checked
    }
}

impl crate::framework::widget::Widget for Checkbox {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let width = area.width.max(4) as usize;
        let height = area.height.max(1) as usize;
        let mut plane = Plane::new(width, height);

        let check_str = if self.checked { "[x]" } else { "[ ]" };
        let full_text = format!("{} {}", check_str, self.label);

        let cell_width = full_text.width().min(width);
        let start_x = (width.saturating_sub(cell_width)) / 2;
        let start_y = height.saturating_sub(1) / 2;

        let fg = if self.checked {
            self.theme.success_fg
        } else {
            self.theme.fg
        };

        for (i, c) in full_text.chars().take(width).enumerate() {
            plane.set_cell(
                (start_x + i) as i32,
                start_y as i32,
                Cell::new(c, Styles::default().with_fg(fg).with_bg(self.theme.bg)),
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
                    cb(self.checked);
                }
                true
            }
            _ => false,
        }
    }
}