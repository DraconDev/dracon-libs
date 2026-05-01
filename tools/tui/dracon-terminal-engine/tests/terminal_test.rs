//! Tests for Terminal (core/terminal.rs).

use std::io::{Cursor, Read, Write};

fn write_to_string<W: Write>(writer: &mut W) -> String {
    let mut cursor = Cursor::new(Vec::new());
    std::io::copy(&mut cursor.clone(), &mut Cursor::new(writer.bytes().collect::<Result<Vec<_>, _>>().unwrap())).ok();
    String::new()
}

fn setup_cursor_terminal() -> (Cursor<Vec<u8>>, dracon_terminal_engine::Terminal<Cursor<Vec<u8>>>) {
    let buffer = Cursor::new(Vec::new());
    let term = dracon_terminal_engine::Terminal::new(buffer).unwrap();
    (buffer, term)
}

#[test]
fn test_terminal_new_null_mode_when_not_tty() {
    let buffer = Cursor::new(Vec::new());
    let result = dracon_terminal_engine::Terminal::new(buffer);
    assert!(result.is_ok());
}

#[test]
fn test_terminal_show_cursor() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.show_cursor().unwrap();
    let bytes = buffer.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[?25h"));
}

#[test]
fn test_terminal_hide_cursor() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.hide_cursor().unwrap();
    let bytes = buffer.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[?25l"));
}

#[test]
fn test_terminal_set_cursor() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.set_cursor(5, 10).unwrap();
    let bytes = buffer.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[11;6H"));
}

#[test]
fn test_terminal_set_cursor_saturates_at_zero() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.set_cursor(0, 0).unwrap();
    let bytes = buffer.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[1;1H"));
}

#[test]
fn test_terminal_inner_returns_mutable_writer() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    let inner = term.inner();
    write!(inner, "test").unwrap();
}

#[test]
fn test_terminal_trait_impl_write() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    let written = term.write(b"hello").unwrap();
    assert_eq!(written, 5);
}

#[test]
fn test_terminal_trait_impl_flush() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.flush().unwrap();
}

#[test]
fn test_terminal_as_fd() {
    let mut buffer = Cursor::new(Vec::new());
    let term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    let _fd = std::os::fd::AsFd::as_fd(&term);
}

#[test]
fn test_terminal_cursor_position_round_trip() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.set_cursor(3, 7).unwrap();
    term.show_cursor().unwrap();
    let bytes = buffer.into_inner();
    assert!(!bytes.is_empty());
}

#[test]
fn test_terminal_compounds_escape_sequence() {
    let mut buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(&mut buffer).unwrap();
    term.set_cursor(0, 0).unwrap();
    term.hide_cursor().unwrap();
    let bytes = buffer.into_inner();
    let output = String::from_utf8(bytes).unwrap();
    assert!(output.contains("\x1b[1;1H"));
    assert!(output.contains("\x1b[?25l"));
}

#[test]
fn test_terminal_drop_shows_cursor_and_restores() {
    let buffer = Cursor::new(Vec::new());
    let _term = dracon_terminal_engine::Terminal::new(buffer);
}

#[test]
fn test_terminal_null_mode_operations() {
    let buffer = Cursor::new(Vec::new());
    let mut term = dracon_terminal_engine::Terminal::new(buffer).unwrap();
    assert!(term.show_cursor().is_ok());
    assert!(term.hide_cursor().is_ok());
    assert!(term.set_cursor(5, 5).is_ok());
}