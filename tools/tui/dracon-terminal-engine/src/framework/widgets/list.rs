use crate::framework::hitzone::{HitZone, HitZoneGroup};
use crate::framework::scroll::{ScrollContainer, ScrollState};
use crate::framework::theme::Theme;
use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

pub struct List<'a, T> {
    items: Vec<T>,
    selected: usize,
    offset: usize,
    visible_count: usize,
    theme: Theme,
    on_select: Box<dyn FnMut(&T) + 'a>,
    item_height: u16,
    width: u16,
}

impl<'a, T: Clone + 'static> List<'a, T> {
    pub fn new<F>(items: Vec<T>, on_select: F) -> Self
    where
        F: FnMut(&T) + 'a,
    {
        Self {
            items,
            selected: 0,
            offset: 0,
            visible_count: 10,
            theme: Theme::default(),
            on_select: Box::new(on_select),
            item_height: 1,
            width: 40,
        }
    }

    pub fn with_theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    pub fn with_item_height(mut self, height: u16) -> Self {
        self.item_height = height;
        self
    }

    pub fn with_width(mut self, width: u16) -> Self {
        self.width = width;
        self
    }

    pub fn selected_index(&self) -> usize {
        self.selected
    }

    pub fn scroll_state(&self) -> ScrollState {
        ScrollState {
            offset: self.offset,
            content_height: self.items.len(),
            viewport_height: self.visible_count,
        }
    }

    pub fn render(&self, area: Rect) -> (Plane, HitZoneGroup<'static, usize>) {
        let mut plane = Plane::new(0, area.width, area.height);
        let visible = self.items.iter().skip(self.offset).take(self.visible_count).enumerate();

        let mut group = HitZoneGroup::new();

        for (i, item) in visible {
            let row = i as u16;
            let item_rect = Rect::new(area.x, area.y + row, self.width, self.item_height);

            let is_selected = self.offset + i == self.selected;
            let bg = if is_selected {
                self.theme.selection_bg
            } else {
                self.theme.bg
            };
            let fg = if is_selected {
                self.theme.selection_fg
            } else {
                self.theme.fg
            };

            let text = format!("{:?}", item);
            let cell = Cell {
                char: ' ',
                fg,
                bg,
                style: if is_selected { Styles::BOLD } else { Styles::empty() },
                transparent: false,
                skip: false,
            };

            for col in 0..self.width.min(area.width) {
                let idx = ((row * area.width) + col) as usize;
                if idx < plane.cells.len() {
                    plane.cells[idx] = cell.clone();
                }
            }

            let label_len = text.len().min(self.width as usize - 1);
            let char_idx = (row * area.width) as usize;
            for (j, ch) in text.chars().take(label_len).enumerate() {
                let idx = char_idx + j + 1;
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                }
            }

            let zone = HitZone::new(self.offset + i, item_rect).on_click(move |_| {});
            group.zones_mut().push(zone);
        }

        (plane, group)
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
    ) -> bool {
        if col >= self.width || row >= self.visible_count as u16 {
            return false;
        }
        let idx = self.offset + row as usize;
        if idx >= self.items.len() {
            return false;
        }
        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.selected = idx;
                if let Some(f) = (&mut self.on_select).as_mut() {
                    f(&self.items[idx]);
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollDown => {
                if self.offset + self.visible_count < self.items.len() {
                    self.offset += 1;
                }
                true
            }
            crate::input::event::MouseEventKind::ScrollUp => {
                if self.offset > 0 {
                    self.offset -= 1;
                }
                true
            }
            _ => false,
        }
    }

    pub fn handle_key(&mut self, key: crate::input::event::KeyEvent) -> bool {
        use crate::input::event::{KeyCode, KeyEventKind};
        if key.kind != KeyEventKind::Press {
            return false;
        }
        match key.code {
            KeyCode::Down => {
                if self.selected + 1 < self.items.len() {
                    self.selected += 1;
                    if self.selected >= self.offset + self.visible_count {
                        self.offset = self.selected - self.visible_count + 1;
                    }
                }
                true
            }
            KeyCode::Up => {
                if self.selected > 0 {
                    self.selected -= 1;
                    if self.selected < self.offset {
                        self.offset = self.selected;
                    }
                }
                true
            }
            KeyCode::Home => {
                self.selected = 0;
                self.offset = 0;
                true
            }
            KeyCode::End => {
                self.selected = self.items.len().saturating_sub(1);
                self.offset = self.items.len().saturating_sub(self.visible_count);
                true
            }
            KeyCode::PageDown => {
                self.selected = (self.selected + self.visible_count).min(self.items.len().saturating_sub(1));
                if self.selected >= self.offset + self.visible_count {
                    self.offset = self.selected.saturating_sub(self.visible_count) + 1;
                }
                true
            }
            KeyCode::PageUp => {
                self.selected = self.selected.saturating_sub(self.visible_count);
                self.offset = self.selected;
                true
            }
            KeyCode::Enter => {
                if let Some(f) = (&mut self.on_select).as_mut() {
                    f(&self.items[self.selected]);
                }
                true
            }
            _ => false,
        }
    }
}