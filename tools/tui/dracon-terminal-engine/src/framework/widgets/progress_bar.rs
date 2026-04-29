//! Progress bar widget.
//!
//! A horizontal progress bar showing completion percentage.

use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct ProgressBar {
    id: WidgetId,
    progress: f32,
    theme: Theme,
}

impl ProgressBar {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            progress: 0.0,
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn set_progress(&mut self, value: f32) {
        self.progress = value.clamp(0.0, 1.0);
    }

    pub fn progress(&self) -> f32 {
        self.progress
    }
}

impl crate::framework::widget::Widget for ProgressBar {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let width = area.width.max(4) as usize;
        let height = area.height.max(1) as usize;
        let mut plane = Plane::new(width, height);

        let fill_width = (self.progress * width as f32).round() as usize;
        let fill_width = fill_width.min(width.saturating_sub(2)).max(1);

        for x in 1..fill_width + 1 {
            plane.set_cell(
                x as i32,
                (height / 2) as i32,
                Cell::new(' ', Styles::default().with_bg(self.theme.primary_fg)),
            );
        }

        let left_bracket = Cell::new('[', Styles::default().with_fg(self.theme.fg));
        let right_bracket = Cell::new(']', Styles::default().with_fg(self.theme.fg));

        plane.set_cell(0, (height / 2) as i32, left_bracket);
        plane.set_cell((width - 1) as i32, (height / 2) as i32, right_bracket);

        plane
    }
}