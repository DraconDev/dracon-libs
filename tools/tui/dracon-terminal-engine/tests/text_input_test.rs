//! Tests for TextInput widget (standalone ratatui widget).

mod common;
use common::make_area;

use dracon_terminal_engine::input::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use dracon_terminal_engine::widgets::input::TextInput;

fn make_event(code: KeyCode) -> Event {
    Event::Key(KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: KeyModifiers::empty(),
    })
}

fn make_event_with_mod(code: KeyCode, mods: KeyModifiers) -> Event {
    Event::Key(KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: mods,
    })
}

#[test]
fn test_text_input_new() {
    let input = TextInput::new();
    assert_eq!(input.value, "");
    assert_eq!(input.cursor_position, 0);
}

#[test]
fn test_text_input_with_value() {
    let input = TextInput::new().with_value("hello");
    assert_eq!(input.value, "hello");
    assert_eq!(input.cursor_position, 5);
}

#[test]
fn test_text_input_with_placeholder() {
    let input = TextInput::new().with_placeholder("Type here...");
    assert_eq!(input.placeholder, "Type here...");
}

#[test]
fn test_text_input_set_value() {
    let mut input = TextInput::new();
    input.set_value("world".to_string());
    assert_eq!(input.value, "world");
    assert_eq!(input.cursor_position, 5);
}

#[test]
fn test_text_input_clear() {
    let mut input = TextInput::new().with_value("test");
    input.clear();
    assert_eq!(input.value, "");
    assert_eq!(input.cursor_position, 0);
}

#[test]
fn test_text_input_handle_char_inserts() {
    let mut input = TextInput::new();
    let event = make_event(KeyCode::Char('a'));
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "a");
    assert_eq!(input.cursor_position, 1);
}

#[test]
fn test_text_input_handle_char_esc_ignored() {
    let mut input = TextInput::new();
    let event = make_event(KeyCode::Char('\x1b'));
    let result = input.handle_event(&event);
    assert!(!result);
}

#[test]
fn test_text_input_handle_backspace() {
    let mut input = TextInput::new().with_value("abc");
    let event = make_event(KeyCode::Backspace);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "ab");
    assert_eq!(input.cursor_position, 2);
}

#[test]
fn test_text_input_handle_backspace_at_start() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 0;
    let event = make_event(KeyCode::Backspace);
    let result = input.handle_event(&event);
    assert!(!result);
    assert_eq!(input.value, "abc");
}

#[test]
fn test_text_input_handle_delete() {
    let mut input = TextInput::new().with_value("abc");
    let event = make_event(KeyCode::Delete);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "bc");
}

#[test]
fn test_text_input_handle_delete_at_end() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 3;
    let event = make_event(KeyCode::Delete);
    let result = input.handle_event(&event);
    assert!(!result);
}

#[test]
fn test_text_input_handle_left() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 2;
    let event = make_event(KeyCode::Left);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 1);
}

#[test]
fn test_text_input_handle_left_at_start() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 0;
    let event = make_event(KeyCode::Left);
    let result = input.handle_event(&event);
    assert!(!result);
}

#[test]
fn test_text_input_handle_right() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 1;
    let event = make_event(KeyCode::Right);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 2);
}

#[test]
fn test_text_input_handle_right_at_end() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 3;
    let event = make_event(KeyCode::Right);
    let result = input.handle_event(&event);
    assert!(!result);
}

#[test]
fn test_text_input_handle_home() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 2;
    let event = make_event(KeyCode::Home);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 0);
}

#[test]
fn test_text_input_handle_end() {
    let mut input = TextInput::new().with_value("abc");
    let event = make_event(KeyCode::End);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 3);
}

#[test]
fn test_text_input_ctrl_a() {
    let mut input = TextInput::new().with_value("abc");
    input.cursor_position = 2;
    let event = make_event_with_mod(KeyCode::Char('a'), KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 0);
}

#[test]
fn test_text_input_ctrl_e() {
    let mut input = TextInput::new().with_value("abc");
    let event = make_event_with_mod(KeyCode::Char('e'), KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.cursor_position, 3);
}

#[test]
fn test_text_input_ctrl_u() {
    let mut input = TextInput::new().with_value("hello");
    input.cursor_position = 3;
    let event = make_event_with_mod(KeyCode::Char('u'), KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "lo");
    assert_eq!(input.cursor_position, 0);
}

#[test]
fn test_text_input_ctrl_k() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 5;
    let event = make_event_with_mod(KeyCode::Char('k'), KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "hello");
}

#[test]
fn test_text_input_ctrl_w() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 10;
    let event = make_event_with_mod(KeyCode::Char('w'), KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
    assert_eq!(input.value, "hello ");
    assert_eq!(input.cursor_position, 6);
}

#[test]
fn test_text_input_ctrl_backspace() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 10;
    let event = make_event_with_mod(KeyCode::Backspace, KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
}

#[test]
fn test_text_input_ctrl_delete() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 5;
    let event = make_event_with_mod(KeyCode::Delete, KeyModifiers::CONTROL);
    let result = input.handle_event(&event);
    assert!(result);
}

#[test]
fn test_text_input_repeat_key_returns_false() {
    let mut input = TextInput::new().with_value("abc");
    let event = Event::Key(KeyEvent {
        kind: KeyEventKind::Repeat,
        code: KeyCode::Char('x'),
        modifiers: KeyModifiers::empty(),
    });
    let result = input.handle_event(&event);
    assert!(!result);
}

#[test]
fn test_text_input_alt_backspace() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 10;
    let event = make_event_with_mod(KeyCode::Backspace, KeyModifiers::ALT);
    let result = input.handle_event(&event);
    assert!(result);
}

#[test]
fn test_text_input_render_placeholder() {
    let input = TextInput::new().with_placeholder("Type here");
    let area = make_area(40, 1);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    TextInput::render(input, area, &mut buf);
}

#[test]
fn test_text_input_render_with_value() {
    let input = TextInput::new().with_value("hello");
    let area = make_area(40, 1);
    let mut buf = ratatui::buffer::Buffer::empty(area);
    TextInput::render(input, area, &mut buf);
}

#[test]
fn test_text_input_delete_word_backwards_at_start() {
    let mut input = TextInput::new().with_value("hello world");
    input.cursor_position = 0;
    let result = input.delete_word_backwards();
    assert!(!result);
    assert_eq!(input.value, "hello world");
}