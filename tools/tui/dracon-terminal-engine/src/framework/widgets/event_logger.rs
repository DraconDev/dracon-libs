//! Event logger widget for displaying input events.
//!
//! Shows a scrolling list of recent input events for debugging.

use std::collections::VecDeque;
use crate::compositor::{Cell, Color, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// A recorded input event for display.
pub struct LoggedEvent {
    /// The timestamp for this event.
    pub timestamp: String,
    /// The description of this event.
    pub description: String,
}

/// A widget that displays a scrolling log of recent input events.
pub struct EventLogger {
    id: WidgetId,
    max_events: usize,
    events: VecDeque<LoggedEvent>,
    theme: Theme,
}

impl EventLogger {
    /// Creates a new event logger with the given ID.
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            max_events: 100,
            events: VecDeque::new(),
            theme: Theme::default(),
        }
    }

    /// Sets the theme for this widget.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the maximum number of events to retain.
    pub fn with_max_events(mut self, max: usize) -> Self {
        self.max_events = max;
        self
    }

    /// Logs an event with a timestamp and description.
    pub fn log(&mut self, timestamp: &str, description: &str) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(LoggedEvent {
            timestamp: timestamp.to_string(),
            description: description.to_string(),
        });
    }

    /// Clears all logged events.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

impl crate::framework::widget::Widget for EventLogger {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 170;

        let width = plane.cells.len() / plane.height as usize;

        let events: Vec<_> = self.events.iter().rev().take(area.height as usize).rev().collect();

        for (i, event) in events.iter().enumerate() {
            let line = format!("{}: {}", event.timestamp, event.description);
            for (j, c) in line.chars().take(width).enumerate() {
                let idx = (i as u16 * plane.width + j as u16) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: c,
                        fg: self.theme.fg,
                        bg: self.theme.bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }
        }

        plane
    }
}