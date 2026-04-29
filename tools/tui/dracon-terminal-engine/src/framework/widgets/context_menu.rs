use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContextAction {
    Open,
    Edit,
    Delete,
    Rename,
    Copy,
    Cut,
    Paste,
    Separator,
    Submenu(Vec<(String, ContextAction)>),
}

pub struct ContextMenu {
    items: Vec<(&'static str, ContextAction)>,
    theme: Theme,
    width: u16,
    height: u16,
}

impl ContextMenu {
    pub fn new(items: Vec<(&'static str, ContextAction)>) -> Self {
        Self {
            items,
            theme: Theme::default(),
            width: 20,
            height: items.len() as u16,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn render_at(&self, screen: Rect, anchor_x: u16, anchor_y: u16) -> (Plane, Vec<HitZone<ContextAction>>, u16, u16) {
        let mut x = anchor_x;
        let mut y = anchor_y;

        if x + self.width > screen.width {
            x = screen.width.saturating_sub(self.width);
        }
        if y + self.height > screen.height {
            y = screen.height.saturating_sub(self.height);
        }

        let mut plane = Plane::new(0, self.width, self.height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 200;

        for cell in &mut plane.cells {
            cell.bg = self.theme.bg;
            cell.fg = self.theme.fg;
        }

        let mut zones = Vec::new();
        let mut row: u16 = 0;

        for (label, action) in &self.items {
            match action {
                ContextAction::Separator => {
                    if row < self.height {
                        let idx = (row * self.width) as usize;
                        if idx < plane.cells.len() {
                            for c in 0..self.width {
                                plane.cells[idx + c as usize].char = '─';
                                plane.cells[idx + c as usize].fg = self.theme.border;
                                plane.cells[idx + c as usize].bg = self.theme.bg;
                            }
                        }
                    }
                }
                _ => {
                    if row < self.height {
                        let rect = Rect::new(x, y + row, self.width, 1);
                        let zone = HitZone::new(*action, x, y + row, self.width, 1);
                        zones.push(zone);

                        let idx = (row * self.width) as usize;
                        if idx < plane.cells.len() {
                            plane.cells[idx].char = ' ';
                            plane.cells[idx].bg = self.theme.bg;
                        }

                        for (j, ch) in label.chars().enumerate() {
                            if j as u16 >= self.width - 1 {
                                break;
                            }
                            let idx = (row * self.width + 2 + j as u16) as usize;
                            if idx < plane.cells.len() {
                                plane.cells[idx].char = ch;
                                plane.cells[idx].fg = self.theme.fg;
                            }
                        }
                    }
                }
            }
            row += 1;
        }

        for col in 0..self.width {
            let top_idx = col as usize;
            if top_idx < plane.cells.len() { plane.cells[top_idx].char = '─'; }
            let bot_idx = ((self.height - 1) * self.width + col) as usize;
            if bot_idx < plane.cells.len() { plane.cells[bot_idx].char = '─'; }
        }
        for r in 1..self.height.saturating_sub(1) {
            let left_idx = (r * self.width) as usize;
            if left_idx < plane.cells.len() { plane.cells[left_idx].char = '│'; }
            let right_idx = (r * self.width + self.width - 1) as usize;
            if right_idx < plane.cells.len() { plane.cells[right_idx].char = '│'; }
        }

        (plane, zones, x, y)
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
        anchor_x: u16,
        anchor_y: u16,
    ) -> Option<ContextAction> {
        if col < anchor_x || col >= anchor_x + self.width || row < anchor_y || row >= anchor_y + self.height {
            return None;
        }

        let local_row = row - anchor_y;

        let mut r: u16 = 0;
        for (_, action) in &self.items {
            if r == local_row {
                if let crate::input::event::MouseEventKind::Down(_) = kind {
                    return Some(*action);
                }
                return None;
            }
            r += 1;
        }
        None
    }
}