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
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;
        let height = plane.height as usize;

        let frame = self.frames[self.current_frame];
        let center_x = width / 2;
        let center_y = height / 2;

        let idx = (center_y as u16 * plane.width + center_x as u16) as usize;
        if idx < plane.cells.len() {
            plane.cells[idx] = Cell::new(
                frame,
                Styles::default()
                    .with_fg(self.theme.primary_fg)
                    .with_bg(self.theme.bg),
            );
        }

        plane
    }

    fn handle_key(&mut self, _key: crate::input::event::KeyEvent) -> bool {
        false
    }
}