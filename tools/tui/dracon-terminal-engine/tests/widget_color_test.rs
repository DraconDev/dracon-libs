//! Tests that verify widget render output contains correct cell colors.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, List, Toggle};
use ratatui::layout::Rect;

fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
    Rect::new(x, y, w, h)
}

// === Button render colors ===

#[test]
fn test_button_render_uses_theme_fg_and_bg() {
    let btn = Button::new("OK", Theme::dark());
    let plane = btn.render(rect(0, 0, 10, 3));

    let theme = Theme::dark();
    for cell in &plane.cells {
        if cell.char != ' ' {
            assert_eq!(cell.fg, theme.fg, "button text should use theme.fg");
            break;
        }
    }
}

#[test]
fn test_button_render_different_theme_colors() {
    let dark_btn = Button::new("OK", Theme::dark());
    let cyberpunk_btn = Button::new("OK", Theme::cyberpunk());

    let dark_plane = dark_btn.render(rect(0, 0, 10, 3));
    let cyberpunk_plane = cyberpunk_btn.render(rect(0, 0, 10, 3));

    let dark_fg = dark_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);
    let cyberpunk_fg = cyberpunk_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);

    assert_ne!(dark_fg, cyberpunk_fg, "same button with different themes should produce different fg colors");
}

#[test]
fn test_button_render_dark_theme_has_white_text() {
    let btn = Button::new("X", Theme::dark());
    let plane = btn.render(rect(0, 0, 5, 3));

    let theme = Theme::dark();
    let cell = plane.cells.iter().find(|c| c.char == 'X').unwrap();
    assert_eq!(cell.fg, theme.fg);
}

// === Checkbox render colors ===

#[test]
fn test_checkbox_render_checked_uses_success_fg() {
    let cb = Checkbox::new("Done", true, Theme::dark());
    let plane = cb.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let checked_char = plane.cells.iter().find(|c| c.char == '✓' || c.char == 'X').unwrap();
    assert_eq!(checked_char.fg, theme.success_fg, "checked checkbox should use success_fg");
}

#[test]
fn test_checkbox_render_unchecked_uses_fg() {
    let cb = Checkbox::new("Done", false, Theme::dark());
    let plane = cb.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let checkbox_char = plane.cells.iter().find(|c| c.char == '[' || c.char == ']').unwrap();
    assert_eq!(checkbox_char.fg, theme.fg, "unchecked checkbox should use theme.fg");
}

#[test]
fn test_checkbox_render_toggle_changes_color() {
    let checked = Checkbox::new("X", true, Theme::dark()).render(rect(0, 0, 10, 3));
    let unchecked = Checkbox::new("X", false, Theme::dark()).render(rect(0, 0, 10, 3));

    let checked_fg = checked.cells.iter().find(|c| c.char != ' ').map(|c| c.fg);
    let unchecked_fg = unchecked.cells.iter().find(|c| c.char != ' ').map(|c| c.fg);

    assert_ne!(checked_fg, unchecked_fg, "checked and unchecked should have different fg colors");
}

// === Toggle render colors ===

#[test]
fn test_toggle_render_on_uses_success_fg() {
    let t = Toggle::new("Power", true, Theme::dark());
    let plane = t.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let on_cell = plane.cells.iter().find(|c| c.char == 'O').unwrap();
    assert_eq!(
        on_cell.fg, theme.success_fg,
        "Toggle ON should use success_fg"
    );
}

#[test]
fn test_toggle_render_off_uses_inactive_fg() {
    let t = Toggle::new("Power", false, Theme::dark());
    let plane = t.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let off_cell = plane.cells.iter().find(|c| c.char == 'F').unwrap();
    assert_eq!(
        off_cell.fg, theme.inactive_fg,
        "Toggle OFF should use inactive_fg"
    );
}

#[test]
fn test_toggle_render_different_themes_different_colors() {
    let dark = Toggle::new("X", true, Theme::dark()).render(rect(0, 0, 10, 3));
    let cyberpunk = Toggle::new("X", true, Theme::cyberpunk()).render(rect(0, 0, 10, 3));

    let dark_color = dark.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);
    let cyberpunk_color = cyberpunk.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);

    assert_ne!(
        dark_color, cyberpunk_color,
        "Toggle ON with dark vs cyberpunk should differ"
    );
}

// === List render colors ===

#[test]
fn test_list_render_selected_item_uses_selection_bg() {
    let items = vec!["item1".to_string(), "item2".to_string(), "item3".to_string()];
    let mut list = List::new(items);
    list.selected_index = Some(1);
    list.theme = Theme::dark();

    let plane = list.render(rect(0, 0, 20, 5));

    let theme = Theme::dark();
    let selected_cells: Vec<_> = plane.cells.iter().filter(|c| c.bg == theme.selection_bg).collect();
    assert!(
        !selected_cells.is_empty(),
        "selected item should have selection_bg"
    );
}

#[test]
fn test_list_render_unselected_item_no_selection_bg() {
    let items = vec!["item1".to_string()];
    let mut list = List::new(items);
    list.selected_index = None;
    list.theme = Theme::dracula();

    let plane = list.render(rect(0, 0, 20, 5));

    let theme = Theme::dracula();
    let selected_cells: Vec<_> = plane.cells.iter().filter(|c| c.bg == theme.selection_bg).collect();
    assert!(
        selected_cells.is_empty(),
        "unselected list should have no selection_bg cells"
    );
}

#[test]
fn test_list_render_selection_changes_with_theme() {
    let items = vec!["a".to_string()];
    let mut dark_list = List::new(items.clone());
    dark_list.selected_index = Some(0);
    dark_list.theme = Theme::dark();

    let mut cyberpunk_list = List::new(items);
    cyberpunk_list.selected_index = Some(0);
    cyberpunk_list.theme = Theme::cyberpunk();

    let dark_plane = dark_list.render(rect(0, 0, 20, 5));
    let cyberpunk_plane = cyberpunk_list.render(rect(0, 0, 20, 5));

    let dark_selection_bg = Theme::dark().selection_bg;
    let cyberpunk_selection_bg = Theme::cyberpunk().selection_bg;

    assert_ne!(
        dark_selection_bg, cyberpunk_selection_bg,
        "dark and cyberpunk have different selection_bg"
    );

    let dark_has_selection = dark_plane.cells.iter().any(|c| c.bg == dark_selection_bg);
    let cyberpunk_has_selection = cyberpunk_plane.cells.iter().any(|c| c.bg == cyberpunk_selection_bg);

    assert!(dark_has_selection);
    assert!(cyberpunk_has_selection);
}

// === Theme vs hardcoded invariant ===

#[test]
fn test_all_widgets_with_theme_produce_non_default_fg_when_theme_specifies() {
    let theme = Theme::cyberpunk();
    let btn = Button::new("X", theme);
    let plane = btn.render(rect(0, 0, 5, 3));

    let text_cell = plane.cells.iter().find(|c| c.char == 'X').unwrap();
    assert!(
        !matches!(text_cell.fg, Color::Reset),
        "widget with explicit theme should not have Reset fg"
    );
}

#[test]
fn test_widget_theme_colors_match_theme_fields() {
    let theme = Theme::dracula();
    let list_items = vec!["a".to_string()];
    let mut list = List::new(list_items);
    list.selected_index = Some(0);
    list.theme = theme;

    let plane = list.render(rect(0, 0, 20, 5));

    let selection_bg = theme.selection_bg;
    let has_selection = plane.cells.iter().any(|c| c.bg == selection_bg);
    assert!(has_selection, "list should render with theme.selection_bg");
}

#[test]
fn test_theme_colors_are_not_white_on_white() {
    let theme = Theme::light();

    assert_ne!(theme.fg, theme.bg, "theme fg and bg should differ");
    assert_ne!(
        theme.selection_fg, theme.selection_bg,
        "selection_fg and selection_bg should differ"
    );
    assert_ne!(
        theme.accent, theme.bg,
        "accent and bg should differ for visibility"
    );
}

#[test]
fn test_checkbox_render_unchecked_bg_is_theme_bg() {
    let theme = Theme::nord();
    let cb = Checkbox::new("Test", false, theme);
    let plane = cb.render(rect(0, 0, 20, 3));

    let checkbox_cells: Vec<_> = plane.cells.iter().filter(|c| c.char == '[' || c.char == ']').collect();
    assert!(!checkbox_cells.is_empty(), "checkbox should render bracket chars");

    for cell in &checkbox_cells {
        assert_eq!(
            cell.bg, theme.bg,
            "checkbox background should be theme.bg"
        );
    }
}
