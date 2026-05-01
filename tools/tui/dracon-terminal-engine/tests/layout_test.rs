//! Tests for standalone layout module (src/layout.rs).

mod common;
use common::make_area;

use dracon_terminal_engine::layout::Stack;
use dracon_terminal_engine::widgets::component::{Bounds, Component};
use dracon_terminal_engine::layout::Orientation;
use dracon_terminal_engine::compositor::Compositor;

struct MockComponent {
    bounds: Bounds,
    name: &'static str,
}

impl MockComponent {
    fn new(name: &'static str) -> Self {
        Self {
            bounds: Bounds::new(0, 0, 10, 1),
            name,
        }
    }

    fn with_bounds(mut self, bounds: Bounds) -> Self {
        self.bounds = bounds;
        self
    }
}

impl Component for MockComponent {
    fn bounds(&self) -> Bounds {
        self.bounds
    }

    fn set_bounds(&mut self, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn render(&self, _compositor: &mut Compositor, bounds: Bounds) {
        self.bounds = bounds;
    }

    fn on_event(&mut self, _event: &dracon_terminal_engine::input::event::Event, _bounds: Bounds) -> bool {
        false
    }
}

#[test]
fn test_centered_rect_creates_centered_area() {
    use dracon_terminal_engine::layout::centered_rect;
    let area = make_area(100, 50);
    let result = centered_rect(50, 50, area);
    assert!(result.x > 0);
    assert!(result.y > 0);
    assert!(result.width < area.width);
    assert!(result.height < area.height);
}

#[test]
fn test_centered_rect_small_percentages() {
    use dracon_terminal_engine::layout::centered_rect;
    let area = make_area(100, 50);
    let result = centered_rect(10, 10, area);
    assert!(result.width < area.width);
    assert!(result.height < area.height);
}

#[test]
fn test_centered_rect_stays_within_bounds() {
    use dracon_terminal_engine::layout::centered_rect;
    let area = make_area(100, 50);
    let result = centered_rect(100, 100, area);
    assert_eq!(result.x, 0);
    assert_eq!(result.y, 0);
    assert_eq!(result.width, area.width);
}

#[test]
fn test_stack_new_horizontal() {
    let stack = Stack::new(Orientation::Horizontal);
    assert_eq!(stack.orientation, Orientation::Horizontal);
    assert!(stack.children.is_empty());
    assert_eq!(stack.spacing, 1);
}

#[test]
fn test_stack_new_vertical() {
    let stack = Stack::new(Orientation::Vertical);
    assert_eq!(stack.orientation, Orientation::Vertical);
    assert!(stack.children.is_empty());
}

#[test]
fn test_stack_with_spacing() {
    let stack = Stack::new(Orientation::Horizontal).with_spacing(5);
    assert_eq!(stack.spacing, 5);
}

#[test]
fn test_stack_add_child() {
    let mut stack = Stack::new(Orientation::Horizontal);
    let mut comp = MockComponent::new("child1");
    stack.add_child(&mut comp);
    assert_eq!(stack.children.len(), 1);
}

#[test]
fn test_stack_multiple_children() {
    let mut stack = Stack::new(Orientation::Horizontal);
    let mut c1 = MockComponent::new("c1");
    let mut c2 = MockComponent::new("c2");
    let mut c3 = MockComponent::new("c3");
    stack.add_child(&mut c1);
    stack.add_child(&mut c2);
    stack.add_child(&mut c3);
    assert_eq!(stack.children.len(), 3);
}

#[test]
fn test_stack_vertical_children() {
    let mut stack = Stack::new(Orientation::Vertical);
    let mut c1 = MockComponent::new("c1");
    let mut c2 = MockComponent::new("c2");
    stack.add_child(&mut c1);
    stack.add_child(&mut c2);
    assert_eq!(stack.children.len(), 2);
}

#[test]
fn test_stack_empty_on_event_returns_false() {
    let stack = Stack::new(Orientation::Horizontal);
    let event = dracon_terminal_engine::input::event::Event::Key(dracon_terminal_engine::input::event::KeyEvent {
        kind: dracon_terminal_engine::input::event::KeyEventKind::Press,
        code: dracon_terminal_engine::input::event::KeyCode::Enter,
        modifiers: dracon_terminal_engine::input::event::KeyModifiers::empty(),
    });
    let result = stack.on_event(&event, Bounds::new(0, 0, 80, 24));
    assert!(!result);
}

#[test]
fn test_stack_with_zero_children_render_does_not_panic() {
    let stack = Stack::new(Orientation::Horizontal);
    let mut compositor = Compositor::new(80, 24);
    stack.render(&mut compositor, Bounds::new(0, 0, 80, 24));
}

#[test]
fn test_stack_spacing_zero() {
    let stack = Stack::new(Orientation::Horizontal).with_spacing(0);
    assert_eq!(stack.spacing, 0);
}

#[test]
fn test_stack_spacing_large() {
    let stack = Stack::new(Orientation::Horizontal).with_spacing(100);
    assert_eq!(stack.spacing, 100);
}

#[test]
fn test_stack_horizontal_sizing() {
    let mut stack = Stack::new(Orientation::Horizontal).with_spacing(0);
    let mut c1 = MockComponent::new("c1");
    let mut c2 = MockComponent::new("c2");
    stack.add_child(&mut c1);
    stack.add_child(&mut c2);
    let mut compositor = Compositor::new(80, 24);
    stack.render(&mut compositor, Bounds::new(0, 0, 80, 24));
}