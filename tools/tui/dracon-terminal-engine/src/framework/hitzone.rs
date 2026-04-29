use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::Widget;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickKind {
    Single,
    Double,
    Triple,
}

pub struct HitZone<'a, T = ()> {
    pub id: T,
    pub rect: Rect,
    on_click: Option<Box<dyn FnMut(ClickKind) + 'a>>,
    on_hover: Option<Box<dyn FnMut(bool) + 'a>>,
    on_drag: Option<Box<dyn FnMut(DragState) + 'a>>,
    on_right_click: Option<Box<dyn FnMut() + 'a>>,
    double_click_timeout: Duration,
    last_click: Option<(Instant, ratatui::layout::Position)>,
    click_count: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DragState {
    Started(ratatui::layout::Position),
    Moved(ratatui::layout::Position),
    Ended(Option<ratatui::layout::Position>),
}

impl<'a, T: 'static> HitZone<'a, T> {
    pub fn new(id: T, rect: Rect) -> Self {
        Self {
            id,
            rect,
            on_click: None,
            on_hover: None,
            on_drag: None,
            on_right_click: None,
            double_click_timeout: Duration::from_millis(500),
            last_click: None,
            click_count: 0,
        }
    }

    pub fn on_click<F>(mut self, f: F) -> Self
    where
        F: FnMut(ClickKind) + 'a,
    {
        self.on_click = Some(Box::new(f));
        self
    }

    pub fn on_hover<F>(mut self, f: F) -> Self
    where
        F: FnMut(bool) + 'a,
    {
        self.on_hover = Some(Box::new(f));
        self
    }

    pub fn on_drag<F>(mut self, f: F) -> Self
    where
        F: FnMut(DragState) + 'a,
    {
        self.on_drag = Some(Box::new(f));
        self
    }

    pub fn on_right_click<F>(mut self, f: F) -> Self
    where
        F: FnMut() + 'a,
    {
        self.on_right_click = Some(Box::new(f));
        self
    }

    pub fn with_double_click_timeout(mut self, ms: u64) -> Self {
        self.double_click_timeout = Duration::from_millis(ms);
        self
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
        modifiers: crate::input::event::KeyModifiers,
    ) {
        let pos = ratatui::layout::Position { x: col, y: row };
        let now = Instant::now();

        match kind {
            crate::input::event::MouseEventKind::Down(btn) => {
                if btn == crate::input::event::MouseButton::Right {
                    if let Some(f) = self.on_right_click.as_mut() {
                        f();
                    }
                    return;
                }
                if btn != crate::input::event::MouseButton::Left {
                    return;
                }

                if let Some((time, prev_pos)) = self.last_click {
                    if time.elapsed() < self.double_click_timeout
                        && (prev_pos.x as i32 - col as i32).abs() <= 1
                        && (prev_pos.y as i32 - row as i32).abs() <= 1
                    {
                        self.click_count += 1;
                    } else {
                        self.click_count = 1;
                    }
                } else {
                    self.click_count = 1;
                }

                self.last_click = Some((now, pos));

                if let Some(f) = self.on_click.as_mut() {
                    let click_kind = match self.click_count {
                        1 => ClickKind::Single,
                        2 => ClickKind::Double,
                        _ => ClickKind::Triple,
                    };
                    f(click_kind);
                }

                if let Some(f) = self.on_drag.as_mut() {
                    f(DragState::Started(pos));
                }
            }
            crate::input::event::MouseEventKind::Drag(_) => {
                if let Some(f) = self.on_drag.as_mut() {
                    f(DragState::Moved(pos));
                }
            }
            crate::input::event::MouseEventKind::Up(_) => {
                if let Some(f) = self.on_drag.as_mut() {
                    f(DragState::Ended(Some(pos)));
                }
            }
            _ => {}
        }
    }

    pub fn handle_hover(&mut self, entered: bool) {
        if let Some(f) = self.on_hover.as_mut() {
            f(entered);
        }
    }
}

pub struct HitZoneGroup<'a, T = ()> {
    zones: Vec<HitZone<'a, T>>,
}

impl<'a, T: 'static> HitZoneGroup<'a, T> {
    pub fn new() -> Self {
        Self { zones: Vec::new() }
    }

    pub fn zone(mut self, zone: HitZone<'a, T>) -> Self {
        self.zones.push(zone);
        self
    }

    pub fn add_row<F>(mut self, id: T, y: u16, width: u16, height: u16, on_click: F) -> Self
    where
        F: FnMut(ClickKind) + 'a,
    {
        let zone = HitZone::new(id, Rect::new(0, y, width, height)).on_click(on_click);
        self.zones.push(zone);
        self
    }

    pub fn zones_mut(&mut self) -> &mut Vec<HitZone<'a, T>> {
        &mut self.zones
    }

    pub fn handle_mouse(
        &mut self,
        kind: crate::input::event::MouseEventKind,
        col: u16,
        row: u16,
        modifiers: crate::input::event::KeyModifiers,
    ) -> bool {
        for zone in self.zones.iter_mut() {
            if zone.rect.contains(ratatui::layout::Position { x: col, y: row }) {
                zone.handle_mouse(kind, col, row, modifiers);
                return true;
            }
        }
        false
    }

    pub fn dispatch(
        &mut self,
        event: &crate::input::event::Event,
    ) -> Option<T> {
        match event {
            crate::input::event::Event::Mouse(me) => {
                for zone in self.zones.iter_mut() {
                    if zone.rect.contains(ratatui::layout::Position {
                        x: me.column,
                        y: me.row,
                    }) {
                        zone.handle_mouse(me.kind.clone(), me.column, me.row, me.modifiers);
                        return Some(zone.id.clone());
                    }
                }
                None
            }
            _ => None,
        }
    }
}

impl<'a, T> Default for HitZoneGroup<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}