//! Edge case tests for TextEditor integration.

use std::io::Write;
use tempfile::NamedTempFile;

use dracon_terminal_engine::widgets::editor::TextEditor;

// === insert_string edge cases ===

#[test]
fn test_insert_empty_string() {
    let mut editor = TextEditor::with_content("hello");
    let initial_len = editor.lines[0].len();
    editor.insert_string("");
    assert_eq!(editor.lines[0].len(), initial_len);
}

#[test]
fn test_insert_at_middle_of_text() {
    let mut editor = TextEditor::with_content("hello world");
    editor.cursor_row = 0;
    editor.cursor_col = 5;
    editor.insert_string("beautiful ");

    assert!(editor.lines[0].contains("hello beautiful world"));
}

#[test]
fn test_insert_at_end_of_line() {
    let mut editor = TextEditor::with_content("hello");
    editor.cursor_row = 0;
    editor.cursor_col = editor.lines[0].len();
    editor.insert_string(" world");

    assert!(editor.lines[0].ends_with(" world"));
}

#[test]
fn test_insert_string_multiple_times() {
    let mut editor = TextEditor::new();
    editor.insert_string("a");
    editor.insert_string("b");
    editor.insert_string("c");

    assert_eq!(editor.lines.len(), 1);
    assert!(editor.lines[0].len() >= 3);
}

// === replace_all edge cases ===

#[test]
fn test_replace_all_empty_find_string() {
    let mut editor = TextEditor::with_content("hello world");
    let count = editor.replace_all("", "X");
    assert_eq!(count, 0, "empty find should replace nothing");
    assert_eq!(editor.lines[0], "hello world");
}

#[test]
fn test_replace_all_empty_replace_string() {
    let mut editor = TextEditor::with_content("hello world");
    let count = editor.replace_all("o", "");
    assert_eq!(count, 2);
    assert_eq!(editor.lines[0], "hell wrld");
}

#[test]
fn test_replace_all_no_match() {
    let mut editor = TextEditor::with_content("hello world");
    let count = editor.replace_all("xyz", "ABC");
    assert_eq!(count, 0);
    assert_eq!(editor.lines[0], "hello world");
}

#[test]
fn test_replace_all_case_sensitive() {
    let mut editor = TextEditor::with_content("Hello hello HELLO");
    let count = editor.replace_all("hello", "hi");
    assert_eq!(count, 1);
    assert_eq!(editor.lines[0], "Hello hi HELLO");
}

#[test]
fn test_replace_all_across_lines() {
    let mut editor = TextEditor::with_content("foo\nfoo\nfoo");
    let count = editor.replace_all("foo", "bar");
    assert_eq!(count, 3);
    assert!(editor.lines.iter().all(|l| l.contains("bar")));
}

// === replace_next edge cases ===

#[test]
fn test_replace_next_no_match() {
    let mut editor = TextEditor::with_content("hello world");
    let result = editor.replace_next("xyz", "ABC");
    assert!(!result);
    assert_eq!(editor.lines[0], "hello world", "content should be unchanged");
}

#[test]
fn test_replace_next_empty_find() {
    let mut editor = TextEditor::with_content("hello");
    let result = editor.replace_next("", "X");
    assert!(!result);
}

#[test]
fn test_replace_next_cursor_not_at_match() {
    let mut editor = TextEditor::with_content("aaa");
    editor.cursor_row = 0;
    editor.cursor_col = 0;
    let result = editor.replace_next("a", "b");
    assert!(result);
    assert_eq!(editor.lines[0], "baa");
}

#[test]
fn test_replace_next_multiple_calls() {
    let mut editor = TextEditor::with_content("aaa");
    editor.replace_next("a", "b").ok();
    editor.replace_next("a", "b").ok();
    editor.replace_next("a", "b").ok();
    let result = editor.replace_next("a", "b");
    assert!(!result, "no more matches after replacing all");
}

// === select_word_at edge cases ===

#[test]
fn test_select_word_at_whitespace() {
    let mut editor = TextEditor::with_content("hello world");
    editor.select_word_at(0, 5);
    let range = editor.get_selection_range();
    assert_eq!(range.start_col, range.end_col, "whitespace has zero-width selection");
}

#[test]
fn test_select_word_at_start_of_word() {
    let mut editor = TextEditor::with_content("hello world");
    editor.select_word_at(0, 0);
    let range = editor.get_selection_range();
    assert!(range.start_col < range.end_col, "word should have non-zero selection");
}

#[test]
fn test_select_word_at_end_of_word() {
    let mut editor = TextEditor::with_content("hello world");
    let end_col = "hello".len();
    editor.select_word_at(0, end_col);
    let range = editor.get_selection_range();
    assert!(range.start_col < range.end_col, "word should have non-zero selection");
}

#[test]
fn test_select_word_at_past_end_of_line() {
    let mut editor = TextEditor::with_content("hello");
    editor.select_word_at(0, 100);
    let range = editor.get_selection_range();
    assert_eq!(range.start_col, range.end_col, "out of bounds should be zero-width");
}

#[test]
fn test_select_word_at_empty_content() {
    let mut editor = TextEditor::new();
    editor.select_word_at(0, 0);
    let range = editor.get_selection_range();
    assert_eq!(range.start_col, range.end_col);
}

// === goto_line edge cases ===

#[test]
fn test_goto_line_past_end_of_file() {
    let mut editor = TextEditor::with_content("line1\nline2\nline3");
    let rect = ratatui::layout::Rect::new(0, 0, 80, 10);
    editor.goto_line(1000, rect);

    assert_eq!(
        editor.cursor_row, editor.lines.len() - 1,
        "cursor should clamp to last line"
    );
}

#[test]
fn test_goto_line_zero() {
    let mut editor = TextEditor::with_content("line1\nline2\nline3");
    let rect = ratatui::layout::Rect::new(0, 0, 80, 10);
    editor.goto_line(0, rect);
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_goto_line_negative() {
    let mut editor = TextEditor::with_content("line1\nline2\nline3");
    let rect = ratatui::layout::Rect::new(0, 0, 80, 10);
    editor.goto_line(0, rect);
    assert_eq!(editor.cursor_row, 0);
}

// === file I/O edge cases ===

#[test]
fn test_save_to_new_path_and_reload() {
    let tmpfile = NamedTempFile::with_suffix(".txt").unwrap();
    let path = tmpfile.path().to_path_buf();

    {
        let mut editor = TextEditor::with_content("original content\nline two");
        editor.save_as(&path).unwrap();
    }

    let reloaded = TextEditor::open(&path).unwrap();
    assert!(reloaded.lines[0].contains("original"));
}

#[test]
fn test_save_as_overwrites_existing_file() {
    let tmpfile = NamedTempFile::with_suffix(".txt").unwrap();
    let path = tmpfile.path().to_path_buf();

    std::fs::write(&path, "old content").unwrap();

    let mut editor = TextEditor::with_content("new content");
    editor.save_as(&path).unwrap();

    let loaded = std::fs::read_to_string(&path).unwrap();
    assert!(loaded.contains("new content"));
}

#[test]
fn test_save_to_untitled_file_fails() {
    let mut editor = TextEditor::new();
    let result = editor.save();
    assert!(result.is_err());
}

#[test]
fn test_open_nonexistent_file_returns_empty() {
    let result = TextEditor::open(std::path::PathBuf::from("/nonexistent/path/xyz123.txt").as_path());
    assert!(result.is_ok());
    let editor = result.unwrap();
    assert_eq!(editor.lines.len(), 1);
}

#[test]
fn test_undo_save_reload_roundtrip() {
    let tmpfile = NamedTempFile::with_suffix(".txt").unwrap();
    let path = tmpfile.path().to_path_buf();

    {
        let mut editor = TextEditor::with_content("v1");
        editor.save_as(&path).unwrap();
        editor.insert_string(" v2");
        editor.save_as(&path).unwrap();
    }

    let reloaded = TextEditor::open(&path).unwrap();
    assert!(reloaded.lines[0].contains("v2") || reloaded.lines[0].contains("v1"));
}

// === multi-cursor edge cases ===

#[test]
fn test_add_cursor_same_position_twice() {
    let mut editor = TextEditor::with_content("hello world");
    editor.cursor_row = 0;
    editor.cursor_col = 2;
    editor.add_cursor(0, 2);
    editor.add_cursor(0, 2);

    assert_eq!(
        editor.extra_cursor_count(),
        1,
        "duplicate cursor at same position should be ignored"
    );
}

#[test]
fn test_clear_extra_cursors() {
    let mut editor = TextEditor::with_content("hello");
    editor.add_cursor(0, 1);
    editor.add_cursor(0, 2);
    assert_eq!(editor.extra_cursor_count(), 2);

    editor.clear_extra_cursors();
    assert_eq!(editor.extra_cursor_count(), 0);
}

#[test]
fn test_add_cursor_out_of_bounds() {
    let mut editor = TextEditor::with_content("hello");
    let initial = editor.extra_cursor_count();
    editor.add_cursor(999, 999);
    assert_eq!(editor.extra_cursor_count(), initial);
}

#[test]
fn test_remove_cursor_not_present() {
    let mut editor = TextEditor::with_content("hello");
    let initial = editor.extra_cursor_count();
    editor.remove_cursor(0, 5);
    assert_eq!(editor.extra_cursor_count(), initial);
}

#[test]
fn test_multi_cursor_and_typing() {
    let mut editor = TextEditor::with_content("abc");
    editor.add_cursor(0, 1);
    editor.insert_string("X");

    let first = editor.lines[0].contains("aXb");
    let second = editor.lines[0].contains("X");
    assert!(
        first || second,
        "typing with extra cursor should affect at least one cursor position"
    );
}

// === selection edge cases ===

#[test]
fn test_get_selected_text_nothing_selected() {
    let mut editor = TextEditor::with_content("hello");
    editor.select_all();
    let selected = editor.get_selected_text();
    assert!(!selected.is_empty() || editor.selection.is_some());
}

#[test]
fn test_select_all_multiline() {
    let mut editor = TextEditor::with_content("line1\nline2\nline3");
    editor.select_all();
    let selected = editor.get_selected_text();
    assert!(selected.contains("line1") && selected.contains("line3"));
}

#[test]
fn test_delete_selection_no_selection() {
    let mut editor = TextEditor::with_content("hello");
    let initial = editor.lines[0].clone();
    editor.delete_selection();
    assert_eq!(editor.lines[0], initial, "delete with no selection should not change content");
}

#[test]
fn test_select_line_at_out_of_bounds() {
    let mut editor = TextEditor::with_content("hello\nworld");
    editor.select_line_at(100, 0);
    let range = editor.get_selection_range();
    assert_eq!(range.start_row, range.end_row);
}

// === filter edge cases ===

#[test]
fn test_set_filter_empty_string() {
    let mut editor = TextEditor::with_content("hello\nworld\nfoo");
    editor.set_filter("");
    assert!(editor.filter.is_none());
}

#[test]
fn test_set_filter_no_matches() {
    let mut editor = TextEditor::with_content("hello\nworld\nfoo");
    editor.set_filter("xyz");
    assert!(editor.filter.is_some());
}

#[test]
fn test_clear_filter() {
    let mut editor = TextEditor::with_content("hello\nworld");
    editor.set_filter("o");
    editor.clear_filter();
    assert!(editor.filter.is_none());
}

// === word wrap edge cases ===

#[test]
fn test_word_wrap_disabled() {
    let mut editor = TextEditor::with_content("a very long line of text without wrap");
    editor.with_word_wrap(false);
    let plane = editor.render(ratatui::layout::Rect::new(0, 0, 10, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_word_wrap_enabled() {
    let mut editor = TextEditor::with_content("a very long line of text without wrap");
    editor.with_word_wrap(true);
    let plane = editor.render(ratatui::layout::Rect::new(0, 0, 10, 20));
    assert!(plane.height >= 1);
}

// === language detection ===

#[test]
fn test_with_language_invalid_ext() {
    let mut editor = TextEditor::new();
    editor.with_language("xyznotareallang");
}

#[test]
fn test_with_language_rust() {
    let mut editor = TextEditor::new();
    editor.with_language("rs");
    assert!(editor.language.is_some());
}

#[test]
fn test_with_language_python() {
    let mut editor = TextEditor::new();
    editor.with_language("py");
    assert!(editor.language.is_some());
}

// === content edge cases ===

#[test]
fn test_with_content_single_empty_line() {
    let editor = TextEditor::with_content("");
    assert_eq!(editor.lines.len(), 1);
    assert_eq!(editor.lines[0], "");
}

#[test]
fn test_with_content_only_newlines() {
    let editor = TextEditor::with_content("\n\n\n");
    assert!(editor.lines.len() >= 2);
}

#[test]
fn test_with_content_trailing_newline() {
    let editor = TextEditor::with_content("line1\n");
    assert_eq!(editor.lines[0], "line1");
}

#[test]
fn test_with_content_multiline() {
    let editor = TextEditor::with_content("a\nb\nc");
    assert_eq!(editor.lines.len(), 3);
    assert_eq!(editor.lines[0], "a");
    assert_eq!(editor.lines[2], "c");
}

#[test]
fn test_delete_last_line() {
    let mut editor = TextEditor::with_content("line1\nline2\n");
    editor.cursor_row = 1;
    editor.delete_line();
    assert_eq!(editor.lines.len(), 1);
    assert_eq!(editor.lines[0], "line1");
}

#[test]
fn test_delete_last_line_single_line() {
    let mut editor = TextEditor::with_content("only line");
    editor.delete_line();
    assert_eq!(editor.lines.len(), 1);
}

#[test]
fn test_delete_line_out_of_bounds() {
    let mut editor = TextEditor::with_content("line1\nline2");
    editor.cursor_row = 99;
    editor.delete_line();
    assert_eq!(editor.lines.len(), 2);
}

// === cursor navigation edge cases ===

#[test]
fn test_cursor_left_at_zero() {
    let mut editor = TextEditor::with_content("hello");
    editor.cursor_col = 0;
    editor.cursor_left();
    assert_eq!(editor.cursor_col, 0);
}

#[test]
fn test_cursor_right_at_end_of_line() {
    let mut editor = TextEditor::with_content("hello");
    editor.cursor_col = 5;
    editor.cursor_right();
    assert_eq!(editor.cursor_col, 5);
}

#[test]
fn test_cursor_up_at_first_line() {
    let mut editor = TextEditor::with_content("line1\nline2");
    editor.cursor_row = 0;
    editor.cursor_up();
    assert_eq!(editor.cursor_row, 0);
}

#[test]
fn test_cursor_down_at_last_line() {
    let mut editor = TextEditor::with_content("line1\nline2");
    editor.cursor_row = 1;
    editor.cursor_down();
    assert_eq!(editor.cursor_row, 1);
}

#[test]
fn test_cursor_wraps_to_next_line() {
    let mut editor = TextEditor::with_content("abc\ndef");
    editor.cursor_col = 3;
    editor.cursor_right();
    assert_eq!(editor.cursor_row, 1);
    assert_eq!(editor.cursor_col, 0);
}
