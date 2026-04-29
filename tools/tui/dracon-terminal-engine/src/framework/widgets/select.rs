//! Select widget for choosing from a dropdown list.
//!
//! A compact widget showing the currently selected item with a dropdown.

use unicode_width::UnicodeWidthStr;
use crate::compositor::{Cell, Plane, Styles};
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

pub struct Select {
    id: WidgetId,
    options: Vec<String>,
    selected: usize,
    expanded: bool,
    theme: Theme,
    on_change: Option<Box<dyn FnMut(&str)>>,
}

impl Select {
    pub fn new(id: WidgetId) -> Self {
        Self {
            id,
            options: Vec::new(),
            selected: 0,
            expanded: false,
            theme: Theme::default(),
            on_change: None,
        }
    }

    pub fn with_options(mut self, options: Vec<String>) -> Self {
        self.options = options;
        self
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn on_change(mut self, f: impl FnMut(&str) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn selected_label(&self) -> Option<&str> {
        self.options.get(self.selected).map(|s| s.as_str())
    }
}

impl crate::framework::widget::Widget for Select {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 10;

        let width = plane.cells.len() / plane.height as usize;

        let display = if let Some(label) = self.selected_label() {
            format!("{} {}", label, if self.expanded { "^" } else { "v" })
        } else {
            "(select)".to_string()
        };

        let _cell_width = display.width().min(width);
        let fg = if self.expanded { self.theme.accent } else { self.theme.fg };

        for (i, c) in display.chars().take(width).enumerate() {
            let idx = (0u16 * plane.width + i as u16) as usize;
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

        if self.expanded {
            for (i, option) in self.options.iter().enumerate() {
                if i + 1 >= plane.height as usize {
                    break;
                }
                let is_selected = i == self.selected;
                let prefix = if is_selected { ">" } else { " " };
                let line = format!("{}{}", prefix, option);

                for (j, c) in line.chars().take(width).enumerate() {
                    let idx = ((i + 1) as u16 * plane.width + j as u16) as usize;
                    if idx < plane.cells.len() {
                        plane.cells[idx] = Cell {
                            char: c,
                            fg: if is_selected { self.theme.accent } else { self.theme.fg },
                            bg: if is_selected { self.theme.selection_bg } else { self.theme.bg },
                            style: if is_selected { Styles::BOLD } else { Styles::empty() },
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
            KeyCode::Enter => {
                self.expanded = !self.expanded;
                true
            }
            KeyCode::Down if self.expanded => {
                if self.selected < self.options.len().saturating_sub(1) {
                    self.selected += 1;
                }
                true
            }
            KeyCode::Up if self.expanded => {
                if self.selected > 0 {
                    self.selected -= 1;
                }
                true
            }
            _ => false,
        }
    }
}