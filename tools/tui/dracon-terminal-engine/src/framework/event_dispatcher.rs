//! Event dispatcher for routing input events to widgets.
//!
//! Provides `EventDispatcher` which routes keyboard/mouse events to
//! widgets via HitZone groups, with capture/bubble phases and focus-aware
//! routing.

use crate::framework::focus::FocusManager;
use crate::framework::hitzone::{HitZone, HitZoneGroup};
use crate::framework::widget::WidgetId;
use crate::input::event::{KeyEvent, KeyModifiers, MouseEventKind};
use ratatui::layout::Rect;
use std::collections::HashMap;

struct DispatchEntry {
    zone: HitZone,
    widget_id: WidgetId,
    capture: bool,
}

pub struct EventDispatcher {
    groups: Vec<HitZoneGroup>,
    entries: Vec<DispatchEntry>,
    focus_manager: Option<&'static mut FocusManager>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            groups: Vec::new(),
            entries: Vec::new(),
            focus_manager: None,
        }
    }

    pub fn with_focus(fm: &'static mut FocusManager) -> Self {
        Self {
            groups: Vec::new(),
            entries: Vec::new(),
            focus_manager: Some(fm),
        }
    }

    pub fn add_zone(&mut self, zone: HitZone, widget_id: WidgetId, capture: bool) {
        self.entries.push(DispatchEntry {
            zone,
            widget_id,
            capture,
        });
    }

    pub fn build_groups(&mut self) {
        self.groups.clear();
        let mut capture_zones = HitZoneGroup::new("capture");
        let mut bubble_zones = HitZoneGroup::new("bubble");

        for entry in self.entries.drain(..) {
            if entry.capture {
                capture_zones.add_zone(entry.zone, entry.widget_id);
            } else {
                bubble_zones.add_zone(entry.zone, entry.widget_id);
            }
        }

        self.groups.push(capture_zones);
        self.groups.push(bubble_zones);
    }

    pub fn dispatch_mouse(
        &self,
        kind: MouseEventKind,
        col: u16,
        row: u16,
        handler: &mut dyn FnMut(WidgetId, MouseEventKind, u16, u16) -> bool,
    ) {
        for group in &self.groups {
            if group.handle_mouse(kind, col, row) {
                return;
            }
        }

        for group in &self.groups {
            for (widget_id, zone) in group.zones() {
                if zone.contains(col, row) {
                    if handler(*widget_id, kind, col, row) {
                        return;
                    }
                }
            }
        }
    }

    pub fn dispatch_key<F>(
        &self,
        key: KeyEvent,
        handler: &mut F,
    ) -> bool
    where
        F: FnMut(WidgetId, KeyEvent) -> bool,
    {
        if let Some(fm) = self.focus_manager {
            if key.modifiers.contains(KeyModifiers::TAB) {
                if key.modifiers.contains(KeyModifiers::SHIFT) {
                    if let Some(id) = fm.tab_prev() {
                        return handler(id, key);
                    }
                } else {
                    if let Some(id) = fm.tab_next() {
                        return handler(id, key);
                    }
                }
            }

            if let Some(focused) = fm.focused() {
                handler(focused, key);
                return true;
            }
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::event::MouseButton;

    fn make_zone(x: u16, y: u16, w: u16, h: u16, id: WidgetId) -> HitZone {
        HitZone::new(
            Rect::new(x, y, w, h),
            id,
            Box::new(|_, _, _| {}),
            Box::new(|_| {}),
            Box::new(|_| {}),
        )
    }

    #[test]
    fn test_dispatch_mouse_capture_first() {
        let dispatcher = EventDispatcher::new();
        let hit = dispatcher.dispatch_mouse(
            MouseEventKind::Press(MouseButton::Left),
            10,
            5,
            &mut |id, _, _, _| {
                assert_eq!(id, WidgetId::new(1));
                true
            },
        );
    }

    #[test]
    fn test_tab_navigation_triggers_focus() {
        let mut fm = FocusManager::new();
        fm.register(WidgetId::new(1), true);
        fm.register(WidgetId::new(2), true);

        let mut dispatcher = EventDispatcher::with_focus(std::ptr::addr_of_mut!(fm));

        let key = KeyEvent {
            code: 9,
            modifiers: KeyModifiers::TAB,
            kind: crate::input::event::KeyKind::Press,
        };

        let mut handled = false;
        dispatcher.dispatch_key(key, &mut |id, _| {
            handled = true;
            true
        });

        assert!(handled);
    }

    #[test]
    fn test_add_zone_and_build() {
        let mut dispatcher = EventDispatcher::new();
        let zone = make_zone(0, 0, 10, 10, WidgetId::new(1));
        dispatcher.add_zone(zone, WidgetId::new(1), true);
        dispatcher.build_groups();
    }
}