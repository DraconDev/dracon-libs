//! Tests for framework Label widget.

mod common;
use common::make_area;

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::label::Label;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::compositor::Styles;

#[test]
fn test_label_new() {
    let label = Label::new("Hello");
    assert_eq!(label.text, "Hello");
    assert!(label.dirty);
}

#[test]
fn test_label_with_id() {
    let id = WidgetId::new(5);
    let label = Label::with_id(id, "Text");
    assert_eq!(label.id, id);
}

#[test]
fn test_label_with_theme() {
    let label = Label::new("test").with_theme(Theme::cyberpunk());
    assert_eq!(label.theme.name, "cyberpunk");
}

#[test]
fn test_label_with_style() {
    let label = Label::new("test").with_style(Styles::BOLD);
    assert_eq!(label.style, Styles::BOLD);
}

#[test]
fn test_label_default_area() {
    let label = Label::new("test");
    let area = label.area();
    assert_eq!(area.width, 40);
    assert_eq!(area.height, 1);
}

#[test]
fn test_label_render_width_matches_area() {
    let label = Label::new("test");
    let area = make_area(20, 3);
    let plane = label.render(area);
    assert_eq!(plane.width, 20);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_label_render_text_chars() {
    let label = Label::new("ABC");
    let area = make_area(20, 3);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'A');
    assert_eq!(plane.cells[1].char, 'B');
    assert_eq!(plane.cells[2].char, 'C');
}

#[test]
fn test_label_render_empty_text() {
    let label = Label::new("");
    let area = make_area(20, 3);
    let plane = label.render(area);
    for cell in &plane.cells {
        assert_eq!(cell.char, '\0');
    }
}

#[test]
fn test_label_render_truncates_long_text() {
    let label = Label::new("This is a very long text that should be truncated");
    let area = make_area(10, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].char, 'T');
    assert_eq!(plane.cells[9].char, 's');
}

#[test]
fn test_label_render_theme_colors() {
    let label = Label::new("C").with_theme(Theme::cyberpunk());
    let area = make_area(10, 1);
    let plane = label.render(area);
    assert_eq!(plane.cells[0].fg, Theme::cyberpunk().fg);
    assert_eq!(plane.cells[0].bg, Theme::cyberpunk().bg);
}

#[test]
fn test_label_render_style_applied() {
    let label = Label::new("B").with_style(Styles::BOLD);
    let area = make_area(10, 1);
    let plane = label.render(area);
    assert!(plane.cells[0].style.contains(Styles::BOLD));
}

#[test]
fn test_label_set_text() {
    let mut label = Label::new("old");
    label.set_text("new");
    assert_eq!(label.text, "new");
    assert!(label.dirty);
}

#[test]
fn test_label_set_text_empty() {
    let mut label = Label::new("old");
    label.set_text("");
    assert_eq!(label.text, "");
}

#[test]
fn test_label_clear_dirty() {
    let mut label = Label::new("test");
    label.clear_dirty();
    assert!(!label.dirty);
}

#[test]
fn test_label_mark_dirty() {
    let mut label = Label::new("test");
    label.clear_dirty();
    label.mark_dirty();
    assert!(label.dirty);
}

#[test]
fn test_label_set_area() {
    let mut label = Label::new("test");
    label.set_area(make_area(5, 1));
    assert_eq!(label.area().width, 5);
    assert!(label.dirty);
}

#[test]
fn test_label_focusable_returns_false() {
    let label = Label::new("test");
    assert!(!label.focusable());
}