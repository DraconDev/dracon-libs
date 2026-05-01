//! Additional integration tests for WidgetContainer and WidgetRegistry.

mod common;

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widget_container::{WidgetContainer, WidgetRegistry};
use dracon_terminal_engine::compositor::Plane;
use std::cell::Cell;
use std::rc::Rc;

struct TestWidget {
    id: WidgetId,
    area: std::cell::Cell<ratatui::layout::Rect>,
    dirty: bool,
    focusable_flag: bool,
}

impl TestWidget {
    fn new(id: u32) -> Self {
        Self {
            id: WidgetId::new(id),
            area: std::cell::Cell::new(ratatui::layout::Rect::new(0, 0, 10, 1)),
            dirty: true,
            focusable_flag: true,
        }
    }
}

impl Widget for TestWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn area(&self) -> ratatui::layout::Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: ratatui::layout::Rect) {
        self.area.set(area);
        self.dirty = true;
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: ratatui::layout::Rect) -> Plane {
        Plane::new(self.id.value() as u16, area.width, area.height)
    }

    fn focusable(&self) -> bool {
        self.focusable_flag
    }

    fn handle_key(&mut self, _key: dracon_terminal_engine::input::event::KeyEvent) -> bool {
        false
    }

    fn handle_mouse(&mut self, _kind: dracon_terminal_engine::input::event::MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }
}

#[test]
fn test_widget_container_new_boxes_widget() {
    let widget = Box::new(TestWidget::new(1));
    let container = WidgetContainer::new(widget);
    assert_eq!(container.id(), WidgetId::new(1));
}

#[test]
fn test_widget_container_render_delegates() {
    let widget = Box::new(TestWidget::new(42));
    let container = WidgetContainer::new(widget);
    let area = ratatui::layout::Rect::new(0, 0, 10, 1);
    let plane = container.render(area);
    assert_eq!(plane.z_index, 42);
}

#[test]
fn test_widget_container_handle_key_delegates() {
    let mut widget = Box::new(TestWidget::new(1));
    let mut container = WidgetContainer::new(widget);
    let key = dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Enter,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    };
    let result = container.handle_key(key);
    assert!(!result);
}

#[test]
fn test_widget_container_widget_accessor() {
    let widget = Box::new(TestWidget::new(5));
    let container = WidgetContainer::new(widget);
    let dyn_widget = container.widget();
    assert_eq!(dyn_widget.id(), WidgetId::new(5));
}

#[test]
fn test_widget_container_widget_mut_accessor() {
    let mut widget = Box::new(TestWidget::new(5));
    let mut container = WidgetContainer::new(widget);
    container.widget_mut().mark_dirty();
}

#[test]
fn test_widget_registry_new_is_empty() {
    let registry = WidgetRegistry::new();
    let count = registry.iter().count();
    assert_eq!(count, 0);
}

#[test]
fn test_widget_registry_register_returns_id() {
    let mut registry = WidgetRegistry::new();
    let widget = Box::new(TestWidget::new(99));
    let returned_id = registry.register(widget);
    assert_eq!(returned_id, WidgetId::new(99));
}

#[test]
fn test_widget_registry_get_existing() {
    let mut registry = WidgetRegistry::new();
    let widget = Box::new(TestWidget::new(1));
    let id = WidgetId::new(1);
    registry.register(widget);
    let found = registry.get(id);
    assert!(found.is_some());
}

#[test]
fn test_widget_registry_get_nonexistent() {
    let registry = WidgetRegistry::new();
    let found = registry.get(WidgetId::new(999));
    assert!(found.is_none());
}

#[test]
fn test_widget_registry_unregister_existing() {
    let mut registry = WidgetRegistry::new();
    let widget = Box::new(TestWidget::new(1));
    let id = WidgetId::new(1);
    registry.register(widget);
    registry.unregister(id);
    assert!(registry.get(id).is_none());
}

#[test]
fn test_widget_registry_unregister_nonexistent() {
    let mut registry = WidgetRegistry::new();
    registry.unregister(WidgetId::new(999));
}

#[test]
fn test_widget_registry_multiple_widgets() {
    let mut registry = WidgetRegistry::new();
    registry.register(Box::new(TestWidget::new(1)));
    registry.register(Box::new(TestWidget::new(2)));
    registry.register(Box::new(TestWidget::new(3)));
    assert_eq!(registry.iter().count(), 3);
}

#[test]
fn test_widget_registry_iter_mut() {
    let mut registry = WidgetRegistry::new();
    registry.register(Box::new(TestWidget::new(1)));
    registry.register(Box::new(TestWidget::new(2)));
    let mut count = 0;
    for _ in registry.iter_mut() {
        count += 1;
    }
    assert_eq!(count, 2);
}

#[test]
fn test_widget_registry_get_mut_existing() {
    let mut registry = WidgetRegistry::new();
    let widget = Box::new(TestWidget::new(1));
    registry.register(widget);
    let found = registry.get_mut(WidgetId::new(1));
    assert!(found.is_some());
}

#[test]
fn test_widget_registry_get_mut_nonexistent() {
    let mut registry = WidgetRegistry::new();
    let found = registry.get_mut(WidgetId::new(999));
    assert!(found.is_none());
}

#[test]
fn test_widget_registry_next_id_increments() {
    let mut registry = WidgetRegistry::new();
    let id1 = registry.next_id();
    let id2 = registry.next_id();
    let id3 = registry.next_id();
    assert_ne!(id1, id2);
    assert_ne!(id2, id3);
}

#[test]
fn test_widget_registry_default() {
    let registry = WidgetRegistry::default();
    assert_eq!(registry.iter().count(), 0);
}

#[test]
fn test_widget_registry_z_order_preserved() {
    let mut registry = WidgetRegistry::new();
    for i in 1..=3 {
        registry.register(Box::new(TestWidget::new(i)));
    }
    let ids: Vec<_> = registry.iter().map(|c| c.id().value()).collect();
    assert_eq!(ids, vec![1, 2, 3]);
}