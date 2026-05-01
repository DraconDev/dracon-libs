//! Tests for HitZone system.

mod common;
use common::make_area;

use dracon_terminal_engine::framework::hitzone::{
    ClickKind, DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyModifiers, MouseButton, MouseEventKind};

#[test]
fn test_hit_zone_new() {
    let zone = HitZone::new(42u32, 5, 10, 20, 5);
    assert_eq!(zone.id, 42);
    assert_eq!(zone.x, 5);
    assert_eq!(zone.y, 10);
    assert_eq!(zone.width, 20);
    assert_eq!(zone.height, 5);
}

#[test]
fn test_hit_zone_contains_inside() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(zone.contains(6, 10));
    assert!(zone.contains(24, 14));
    assert!(zone.contains(5, 10));
}

#[test]
fn test_hit_zone_contains_outside() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(!zone.contains(4, 10));
    assert!(!zone.contains(25, 10));
    assert!(!zone.contains(5, 9));
    assert!(!zone.contains(5, 15));
}

#[test]
fn test_hit_zone_contains_on_edge() {
    let zone = HitZone::new(1u32, 5, 10, 20, 5);
    assert!(zone.contains(24, 10));
    assert!(zone.contains(5, 14));
    assert!(!zone.contains(25, 10));
    assert!(!zone.contains(5, 15));
}

#[test]
fn test_hit_zone_on_click_callback() {
    let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(|kind| {
        assert_eq!(kind, ClickKind::Single);
    });
    zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
}

#[test]
fn test_hit_zone_on_right_click() {
    let mut called = false;
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_right_click(|| {
            called = true;
        });
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Right), 10, 12, KeyModifiers::empty());
    }
    assert!(called);
}

#[test]
fn test_hit_zone_dispatch_mouse_returns_id() {
    let mut zone = HitZone::new(99u32, 5, 10, 20, 5).on_click(|_| {});
    let result = zone.dispatch_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
    assert_eq!(result, Some(99));
}

#[test]
fn test_hit_zone_dispatch_mouse_outside_returns_none() {
    let mut zone = HitZone::new(99u32, 5, 10, 20, 5).on_click(|_| {});
    let result = zone.dispatch_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100, KeyModifiers::empty());
    assert_eq!(result, None);
}

#[test]
fn test_hit_zone_drag_start() {
    let mut called = false;
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5).on_drag_start(|state| {
            if let DragState::Started { x, y } = state {
                assert_eq!(*x, 10);
                assert_eq!(*y, 12);
            }
            called = true;
        });
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
    }
    assert!(called);
}

#[test]
fn test_hit_zone_drag_move() {
    let mut positions = Vec::new();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
            .on_drag_start(|_| {})
            .on_drag_move(|state| {
                if let DragState::Moved { x, y } = state {
                    positions.push((*x, *y));
                }
            });
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
        zone.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 15, 13, KeyModifiers::empty());
        zone.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 20, 14, KeyModifiers::empty());
    }
    assert_eq!(positions.len(), 2);
    assert_eq!(positions[0], (15, 13));
}

#[test]
fn test_hit_zone_drag_end() {
    let mut ended = false;
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
            .on_drag_start(|_| {})
            .on_drag_end(|state| {
                if let DragState::Ended { x, y } = state {
                    assert_eq!(*x, 20);
                    assert_eq!(*y, 14);
                }
                ended = true;
            });
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
        zone.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 15, 13, KeyModifiers::empty());
        zone.handle_mouse(MouseEventKind::Up(MouseButton::Left), 20, 14, KeyModifiers::empty());
    }
    assert!(ended);
}

#[test]
fn test_hit_zone_double_click_detection() {
    let mut click_kinds = Vec::new();
    {
        let mut zone = HitZone::new(1u32, 5, 10, 20, 5)
            .double_click_timeout(std::time::Duration::from_millis(500))
            .on_click(|kind| {
                click_kinds.push(kind);
            });
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
        zone.handle_mouse(MouseEventKind::Down(MouseButton::Left), 10, 12, KeyModifiers::empty());
    }
    assert_eq!(click_kinds.len(), 2);
    assert_eq!(click_kinds[0], ClickKind::Single);
}

#[test]
fn test_hit_zone_group_new() {
    let group: HitZoneGroup<u32> = HitZoneGroup::new();
    assert!(group.zones().is_empty());
}

#[test]
fn test_hit_zone_group_add_zone() {
    let mut group = HitZoneGroup::new();
    let zone = HitZone::new(1u32, 5, 10, 20, 5).on_click(|_| {});
    group.zones_mut().push(zone);
    assert_eq!(group.zones().len(), 1);
}

#[test]
fn test_hit_zone_group_builder() {
    let group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(|_| {}))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    assert_eq!(group.zones().len(), 2);
}

#[test]
fn test_hit_zone_group_dispatch_finds_first() {
    let mut group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(|_| {}))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    let result = group.dispatch_mouse(MouseEventKind::Down(MouseButton::Left), 35, 12, KeyModifiers::empty());
    assert_eq!(result, Some(2));
}

#[test]
fn test_hit_zone_group_dispatch_miss() {
    let mut group = HitZoneGroup::new()
        .zone(HitZone::new(1u32, 5, 10, 20, 5).on_click(|_| {}))
        .zone(HitZone::new(2u32, 30, 10, 20, 5).on_click(|_| {}));
    let result = group.dispatch_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100, KeyModifiers::empty());
    assert_eq!(result, None);
}

#[test]
fn test_hit_zone_group_add_row() {
    let mut group = HitZoneGroup::new();
    group.add_row(1u32, 5, 80, |_| {});
    assert_eq!(group.zones().len(), 1);
    assert_eq!(group.zones()[0].x, 0);
    assert_eq!(group.zones()[0].y, 5);
    assert_eq!(group.zones()[0].width, 80);
    assert_eq!(group.zones()[0].height, 1);
}

#[test]
fn test_scoped_zone_new() {
    let zone = ScopedZone::new("id", 5, 10, 20, 5);
    assert_eq!(zone.id, "id");
    assert_eq!(zone.x, 5);
}

#[test]
fn test_scoped_zone_contains() {
    let zone = ScopedZone::new("id", 5, 10, 20, 5);
    assert!(zone.contains(6, 10));
    assert!(!zone.contains(100, 100));
}

#[test]
fn test_scoped_zone_registry_new() {
    let registry: ScopedZoneRegistry<u32> = ScopedZoneRegistry::new();
    assert!(registry.zones().is_empty());
}

#[test]
fn test_scoped_zone_registry_register() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(1u32, 5, 10, 20, 5);
    assert_eq!(registry.zones().len(), 1);
}

#[test]
fn test_scoped_zone_registry_dispatch() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    let result = registry.dispatch(10, 12);
    assert_eq!(result, Some(42));
}

#[test]
fn test_scoped_zone_registry_dispatch_miss() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    let result = registry.dispatch(100, 100);
    assert_eq!(result, None);
}

#[test]
fn test_scoped_zone_registry_clear() {
    let mut registry = ScopedZoneRegistry::new();
    registry.register(42u32, 5, 10, 20, 5);
    registry.clear();
    assert!(registry.zones().is_empty());
}

#[test]
fn test_click_kind_variants() {
    assert_eq!(ClickKind::Single, ClickKind::Single);
    assert_eq!(ClickKind::Double, ClickKind::Double);
    assert_eq!(ClickKind::Triple, ClickKind::Triple);
    assert_ne!(ClickKind::Single, ClickKind::Double);
}

#[test]
fn test_drag_state_variants() {
    let started = DragState::Started { x: 1, y: 2 };
    let moved = DragState::Moved { x: 3, y: 4 };
    let ended = DragState::Ended { x: 5, y: 6 };
    assert_eq!(started.x, 1);
    assert_eq!(moved.y, 4);
    assert_eq!(ended.x, 5);
}