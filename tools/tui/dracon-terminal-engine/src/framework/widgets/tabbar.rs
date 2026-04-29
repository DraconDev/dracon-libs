use crate::framework::hitzone::HitZone;
use crate::framework::theme::Theme;
use crate::compositor::{Cell, Color, Plane, Styles};
use ratatui::layout::Rect;

pub struct TabBar<'a> {
    tabs: Vec<&'a str>,
    active: usize,
    theme: Theme,
    width: u16,
}

impl<'a> TabBar<'a> {
    pub fn new(tabs: Vec<&'a str>) -> Self {
        Self {
            tabs,
            active: 0,
            theme: Theme::default(),
            width: 80,
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

    pub fn active(&self) -> usize {
        self.active
    }

    pub fn set_active(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.active = idx;
        }
    }

    pub fn render(&self, area: Rect) -> (Plane, Vec<HitZone<'static, usize>>) {
        let mut plane = Plane::new(0, area.width, area.height);
        let mut zones = Vec::new();
        let tab_width = (area.width / self.tabs.len() as u16).max(1);

        for (i, tab) in self.tabs.iter().enumerate() {
            let x = (i as u16) * tab_width;
            let rect = Rect::new(x, area.y, tab_width, area.height);
            let is_active = i == self.active;

            let bg = if is_active { self.theme.active_bg } else { self.theme.bg };
            let fg = if is_active { self.theme.accent } else { self.theme.inactive_fg };
            let style = if is_active { Styles::BOLD | Styles::UNDERLINE } else { Styles::empty() };

            for col in 0..tab_width {
                let idx = (col as usize) + (0usize);
                if idx < plane.cells.len() {
                    plane.cells[idx] = Cell {
                        char: ' ',
                        fg,
                        bg,
                        style: Styles::empty(),
                        transparent: false,
                        skip: false,
                    };
                }
            }

            let label_len = tab.len().min(tab_width as usize - 2);
            let start_col = if tab_width > 2 { 1 } else { 0 };
            for (j, ch) in tab.chars().take(label_len).enumerate() {
                let idx = ((start_col + j) as usize);
                if idx < plane.cells.len() {
                    plane.cells[idx].char = ch;
                    plane.cells[idx].fg = fg;
                    plane.cells[idx].bg = bg;
                    plane.cells[idx].style = style;
                }
            }

            let mut zone = HitZone::new(i, rect);
            zone.on_click = Some(Box::new(move |_| {}));
            zones.push(zone);
        }

        (plane, zones)
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        _row: u16,
    ) -> bool {
        let tab_width = (self.width / self.tabs.len() as u16).max(1);
        let idx = col / tab_width;
        if idx >= self.tabs.len() as u16 {
            return false;
        }

        match kind {
            crate::input::event::MouseEventKind::Down(crate::input::event::MouseButton::Left) => {
                self.active = idx as usize;
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
            KeyCode::Left => {
                if self.active > 0 {
                    self.active -= 1;
                }
                true
            }
            KeyCode::Right => {
                if self.active + 1 < self.tabs.len() {
                    self.active += 1;
                }
                true
            }
            _ => false,
        }
    }
}