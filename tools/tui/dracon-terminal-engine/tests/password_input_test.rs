//! Tests for PasswordInput widget.

mod common;
use common::{make_area, make_key, rect};

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::PasswordInput;
use std::cell::Cell;
use std::rc::Rc;

#[test]
fn test_password_input_new() {
    let id = WidgetId::new(1);
    let input = PasswordInput::new(id);
    assert_eq!(input.id(), id);
}

#[test]
fn test_password_input_id_method() {
    let id = WidgetId::new(42);
    let input = PasswordInput::new(id);
    assert_eq!(input.id(), id);
}

#[test]
fn test_password_input_set_id() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let id = WidgetId::new(99);
    input.set_id(id);
    assert_eq!(input.id(), id);
}

#[test]
fn test_password_input_default_area() {
    let input = PasswordInput::new(WidgetId::new(1));
    let area = input.area();
    assert_eq!(area.width, 30);
    assert_eq!(area.height, 1);
}

#[test]
fn test_password_input_render_size() {
    let input = PasswordInput::new(WidgetId::new(1));
    let area = make_area(40, 1);
    let plane = input.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 1);
    assert_eq!(plane.z_index, 10);
}

#[test]
fn test_password_input_render_placeholder_when_empty() {
    let input = PasswordInput::new(WidgetId::new(1));
    let area = make_area(40, 1);
    let plane = input.render(area);
    assert!(plane.cells.iter().any(|c| c.char == 'P'));
}

#[test]
fn test_password_input_with_theme() {
    use dracon_terminal_engine::framework::theme::Theme;
    let input = PasswordInput::new(WidgetId::new(1)).with_theme(Theme::cyberpunk());
    let area = make_area(40, 1);
    let plane = input.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_password_input_with_mask_char() {
    let input = PasswordInput::new(WidgetId::new(1)).with_mask_char('#');
    let area = make_area(40, 1);
    let plane = input.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_password_input_with_placeholder() {
    let input = PasswordInput::new(WidgetId::new(1)).with_placeholder("Enter password");
    let area = make_area(40, 1);
    let plane = input.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_password_input_clear() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    input.clear();
    assert_eq!(input.password(), "");
}

#[test]
fn test_password_input_password_getter() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('s')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('e')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('c')));
    assert_eq!(input.password(), "sec");
}

#[test]
fn test_password_input_handle_key_enter_triggers_callback() {
    let result = Rc::new(Cell::new(None));
    let result_clone = result.clone();
    let mut input = PasswordInput::new(WidgetId::new(1)).on_submit(move |pwd| {
        result_clone.set(Some(pwd.to_string()));
    });
    input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Enter));
    assert!(result.get().is_some());
}

#[test]
fn test_password_input_handle_key_char_inserts() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    assert!(handled);
    assert!(!input.password().is_empty());
}

#[test]
fn test_password_input_handle_key_backspace() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Backspace));
    assert_eq!(input.password(), "a");
}

#[test]
fn test_password_input_handle_key_left() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Left));
    assert!(handled);
}

#[test]
fn test_password_input_handle_key_right() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Right));
    assert!(handled);
}

#[test]
fn test_password_input_handle_key_delete() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Delete));
    assert!(handled);
}

#[test]
fn test_password_input_handle_key_home() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Home));
    assert!(handled);
}

#[test]
fn test_password_input_handle_key_end() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('a')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('b')));
    let handled = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::End));
    assert!(handled);
}

#[test]
fn test_password_input_handle_mouse() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('h')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('e')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('l')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('l')));
    let _ = input.handle_key(make_key(dracon_terminal_engine::input::event::KeyCode::Char('o')));
    let result = input.handle_mouse(dracon_terminal_engine::input::event::MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left), 2, 0);
    assert!(result);
}

#[test]
fn test_password_input_handle_mouse_out_of_bounds() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    let result = input.handle_mouse(dracon_terminal_engine::input::event::MouseEventKind::Down(dracon_terminal_engine::input::event::MouseButton::Left), 100, 0);
    assert!(!result);
}

#[test]
fn test_password_input_cursor_position() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    input.set_area(rect(5, 5, 30, 1));
    let pos = input.cursor_position();
    assert!(pos.is_some());
}

#[test]
fn test_password_input_set_area() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    input.set_area(rect(10, 20, 50, 2));
    let area = input.area();
    assert_eq!(area.x, 10);
    assert_eq!(area.y, 20);
    assert_eq!(area.width, 50);
    assert_eq!(area.height, 2);
}

#[test]
fn test_password_input_needs_render() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    assert!(input.needs_render());
    input.clear_dirty();
    assert!(!input.needs_render());
}

#[test]
fn test_password_input_mark_dirty() {
    let mut input = PasswordInput::new(WidgetId::new(1));
    input.clear_dirty();
    input.mark_dirty();
    assert!(input.needs_render());
}