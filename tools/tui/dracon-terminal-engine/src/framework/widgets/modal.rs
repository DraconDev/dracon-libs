//! Modal dialog widget.

use unicode_width::UnicodeWidthStr;

use crate::compositor::{Plane, Styles};
use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use ratatui::layout::Rect;

/// Result returned when the user clicks a button in a modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModalResult {
    /// User confirmed (e.g. OK button).
    Confirm,
    /// User cancelled.
    Cancel,
    /// A custom button with the given identifier.
    Custom(u8),
}

/// A centered modal dialog with a title, optional buttons, and a border.
pub struct Modal<'a> {
    id: WidgetId,
    title: &'a str,
    width: u16,
    height: u16,
    theme: Theme,
    buttons: Vec<(&'a str, ModalResult)>,
    area: std::cell::Cell<Rect>,
}

impl<'a> Modal<'a> {
    /// Creates a new `Modal` with the given title and default OK/Cancel buttons.
    pub fn new(title: &'a str) -> Self {
        Self {
            id: WidgetId::default_id(),
            title,
            width: 40,
            height: 5,
            theme: Theme::default(),
            buttons: vec![("OK", ModalResult::Confirm), ("Cancel", ModalResult::Cancel)],
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 5)),
        }
    }

    /// Creates a new `Modal` with the given widget ID and title.
    pub fn new_with_id(id: WidgetId, title: &'a str) -> Self {
        Self {
            id,
            title,
            width: 40,
            height: 5,
            theme: Theme::default(),
            buttons: vec![("OK", ModalResult::Confirm), ("Cancel", ModalResult::Cancel)],
            area: std::cell::Cell::new(Rect::new(0, 0, 40, 5)),
        }
    }

    /// Sets the width and height of the modal.
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the theme for rendering.
    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Sets the button label/result pairs.
    pub fn with_buttons(mut self, buttons: Vec<(&'a str, ModalResult)>) -> Self {
        self.buttons = buttons;
        self
    }
}

impl<'a> crate::framework::widget::Widget for Modal<'a> {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        100
    }

    fn render(&self, area: Rect) -> Plane {
        let x = (area.width.saturating_sub(self.width)) / 2;
        let y = (area.height.saturating_sub(self.height)) / 2;

        let mut plane = Plane::new(0, self.width, self.height);
        plane.x = x;
        plane.y = y;
        plane.z_index = 100;

        for cell in &mut plane.cells {
            cell.bg = self.theme.bg;
            cell.fg = self.theme.fg;
        }

        let border_char: char = '─';
        for col in 0..self.width {
            let idx = col as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = border_char; }
            let idx = ((self.height - 1) * self.width + col) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '─'; }
        }
        for row in 1..self.height.saturating_sub(1) {
            let idx = (row * self.width) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '│'; }
            let idx = (row * self.width + self.width - 1) as usize;
            if idx < plane.cells.len() { plane.cells[idx].char = '│'; }
        }

        let title_len = self.title.width().min((self.width as usize).saturating_sub(4));
        let title_start = (self.width as usize - title_len) / 2;
        for (i, ch) in self.title.chars().take(title_len).enumerate() {
            let idx = (1 + title_start + i) as usize;
            if idx < plane.cells.len() {
                plane.cells[idx].char = ch;
                plane.cells[idx].style = Styles::BOLD;
                plane.cells[idx].fg = self.theme.accent;
            }
        }

        let btn_width: u16 = 8;
        let total_btn_width = btn_width * self.buttons.len() as u16 + (self.buttons.len() as u16 - 1);
        let btn_start = (self.width.saturating_sub(total_btn_width)) / 2;
        let btn_y = self.height - 2;

        for (i, (label, _result)) in self.buttons.iter().enumerate() {
            let bx = btn_start + (i as u16) * (btn_width + 1);

            let bg = self.theme.active_bg;
            let fg = self.theme.fg;
            for col in 0..btn_width {
                let col_idx = btn_y as usize * self.width as usize + bx as usize + col as usize;
                if col_idx < plane.cells.len() {
                    plane.cells[col_idx].bg = bg;
                    plane.cells[col_idx].fg = fg;
                    plane.cells[col_idx].char = ' ';
                }
            }

            let label_len = label.width().min((btn_width as usize).saturating_sub(2));
            let label_start = (btn_width as usize - label_len) / 2;
            for (j, ch) in label.chars().take(label_len).enumerate() {
                let label_idx = (btn_y as usize) * (self.width as usize) + (bx as usize) + (label_start as usize) + j;
                if label_idx < plane.cells.len() {
                    plane.cells[label_idx].char = ch;
                    plane.cells[label_idx].style = Styles::BOLD;
                }
            }

            let _zone = HitZone::new(*_result, bx, btn_y, btn_width, 1);
        }

        plane
    }

    fn handle_mouse(&mut self, kind: crate::input::event::MouseEventKind, col: u16, row: u16) -> bool {
        let screen = self.area.get();
        let x = (screen.width.saturating_sub(self.width)) / 2;
        let y = (screen.height.saturating_sub(self.height)) / 2;

        if col < x || col >= x + self.width || row < y || row >= y + self.height {
            return false;
        }

        let local_col = col - x;
        let local_row = row - y;

        let btn_width: u16 = 8;
        let total_btn_width = btn_width * self.buttons.len() as u16 + (self.buttons.len() as u16 - 1);
        let btn_start = (self.width.saturating_sub(total_btn_width)) / 2;
        let btn_y = self.height - 2;

        for (i, (_, _result)) in self.buttons.iter().enumerate() {
            let bx = btn_start + (i as u16) * (btn_width + 1);
            let in_btn = local_col >= bx && local_col < bx + btn_width && local_row == btn_y;

            if in_btn {
                if let crate::input::event::MouseEventKind::Down(_) = kind {
                    return true;
                }
            }
        }

        false
    }
}