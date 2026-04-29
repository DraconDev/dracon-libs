//! Spinner widget.
//!
//! An animated loading spinner with configurable frame sequence.

use std::time::{Duration, Instant};
use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct Spinner {
    id: WidgetId,
    frames: Vec<char>,
    current_frame: usize,
    last_update: Instant,
    theme: Theme,
}

impl Spinner {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            frames: vec!['|', '/', '-', '\\'],
            current_frame: 0,
            last_update: Instant::now(),
            theme: Theme::default(),
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_frames(mut self, frames: Vec<char>) -> Self {
        if !frames.is_empty() {
            self.frames = frames;
        }
        self
    }

    pub fn tick(&mut self) {
        let now = Instant::now();
        if now.duration_since(self.last_update) >= Duration::from_millis(100) {
            self.current_frame = (self.current_frame + 1) % self.frames.len();
            self.last_update = now;
        }
    }

    pub fn current_frame(&self) -> char {
        self.frames[self.current_frame]
    }
}

impl crate::framework::widget::Widget for Spinner {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let width = area.width.max(1) as usize;
        let height = area.height.max(1) as usize;
        let mut plane = Plane::new(width, height);

        let frame = self.frames[self.current_frame];
        let center_x = width / 2;
        let center_y = height / 2;

        let cell = Cell::new(
            frame,
            Styles::default()
                .with_fg(self.theme.primary_fg)
                .with_bg(self.theme.bg),
        );
        plane.set_cell(center_x as i32, center_y as i32, cell);

        plane
    }

    fn handle_key(&mut self, _key: crate::input::event::KeyEvent) -> bool {
        false
    }
}