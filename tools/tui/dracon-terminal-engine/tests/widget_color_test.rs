//! Tests that verify widget render output contains correct cell colors.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, List, Toggle};
use ratatui::layout::Rect;

fn rect(x: u16, y: u16, w: u16, h: u16) -> Rect {
    Rect::new(x, y, w, h)
}

// === Button render colors ===

#[test]
fn test_button_render_uses_theme_fg_and_bg() {
    let btn = Button::new("OK").with_theme(Theme::dark());
    let plane = btn.render(rect(0, 0, 10, 3));

    let theme = Theme::dark();
    let text_cell = plane.cells.iter().find(|c| c.char == 'O');
    assert!(text_cell.is_some(), "button should render text");
    assert_eq!(text_cell.unwrap().fg, theme.fg, "button text should use theme.fg");
}

#[test]
fn test_button_render_different_theme_colors() {
    let dark_btn = Button::new("OK").with_theme(Theme::dark());
    let cyberpunk_btn = Button::new("OK").with_theme(Theme::cyberpunk());

    let dark_plane = dark_btn.render(rect(0, 0, 10, 3));
    let cyberpunk_plane = cyberpunk_btn.render(rect(0, 0, 10, 3));

    let dark_fg = dark_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);
    let cyberpunk_fg = cyberpunk_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);

    assert_ne!(dark_fg, cyberpunk_fg, "same button with different themes should produce different fg colors");
}

#[test]
fn test_button_render_dark_theme_has_white_text() {
    let btn = Button::new("X").with_theme(Theme::dark());
    let plane = btn.render(rect(0, 0, 5, 3));

    let theme = Theme::dark();
    let cell = plane.cells.iter().find(|c| c.char == 'X').unwrap();
    assert_eq!(cell.fg, theme.fg);
}

// === Checkbox render colors ===

#[test]
fn test_checkbox_render_checked_uses_success_fg() {
    let cb = Checkbox::new(WidgetId::default_id(), "Done").with_theme(Theme::dark());
    let mut cb = cb;
    cb.check();
    let plane = cb.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let checked_char = plane.cells.iter().find(|c| c.char == '✓' || c.char == 'X').unwrap();
    assert_eq!(checked_char.fg, theme.success_fg, "checked checkbox should use success_fg");
}

#[test]
fn test_checkbox_render_unchecked_uses_fg() {
    let cb = Checkbox::new(WidgetId::default_id(), "Done").with_theme(Theme::dark());
    let plane = cb.render(rect(0, 0, 20, 3));

    let theme = Theme::dark();
    let checkbox_char = plane.cells.iter().find(|c| c.char == '[' || c.char == ']').unwrap();
    assert_eq!(checkbox_char.fg, theme.fg, "unchecked checkbox should use theme.fg");
}

#[test]
fn test_checkbox_render_toggle_changes_color() {
    let mut checked = Checkbox::new(WidgetId::default_id(), "X").with_theme(Theme::dark());
    checked.check();
    let checked_plane = checked.render(rect(0, 0, 10, 3));

    let unchecked = Checkbox::new(WidgetId::default_id(), "X").with_theme(Theme::dark());
    let unchecked_plane = unchecked.render(rect(0, 0, 10, 3));

    let checked_fg = checked_plane.cells.iter().find(|c| c.char != ' ').map(|c| c.fg);
    let unchecked_fg = unchecked_plane.cells.iter().find(|c| c.char != ' ').map(|c| c.fg);

    assert_ne!(checked_fg, unchecked_fg, "checked and unchecked should have different fg colors");
}

// === Toggle render colors ===

#[test]
fn test_toggle_render_on_uses_success_fg() {
    let mut t = Toggle::new(WidgetId::default_id(), "Power").with_theme(Theme::dark());
    t.toggle();
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
    let t = Toggle::new(WidgetId::default_id(), "Power").with_theme(Theme::dark());
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
    let mut dark = Toggle::new(WidgetId::default_id(), "X").with_theme(Theme::dark());
    dark.toggle();
    let dark_plane = dark.render(rect(0, 0, 10, 3));

    let mut cyberpunk = Toggle::new(WidgetId::default_id(), "X").with_theme(Theme::cyberpunk());
    cyberpunk.toggle();
    let cyberpunk_plane = cyberpunk.render(rect(0, 0, 10, 3));

    let dark_color = dark_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);
    let cyberpunk_color = cyberpunk_plane.cells.iter().find(|c| c.char == 'O').map(|c| c.fg);

    assert_ne!(
        dark_color, cyberpunk_color,
        "Toggle ON with dark vs cyberpunk should differ"
    );
}

// === Theme vs hardcoded invariant ===

#[test]
fn test_all_widgets_with_theme_produce_non_default_fg_when_theme_specifies() {
    let theme = Theme::cyberpunk();
    let btn = Button::new("X").with_theme(theme);
    let plane = btn.render(rect(0, 0, 5, 3));

    let text_cell = plane.cells.iter().find(|c| c.char == 'X').unwrap();
    assert!(
        !matches!(text_cell.fg, Color::Reset),
        "widget with explicit theme should not have Reset fg"
    );
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

// === Button toggle with theme ===

#[test]
fn test_button_render_bracket_chars_present() {
    let btn = Button::new("A").with_theme(Theme::dracula());
    let plane = btn.render(rect(0, 0, 10, 3));

    let bracket_chars: Vec<_> = plane.cells.iter().filter(|c| c.char == '[' || c.char == ']').collect();
    assert!(!bracket_chars.is_empty(), "button should render bracket chars");
}

#[test]
fn test_checkbox_render_bracket_chars_present() {
    let cb = Checkbox::new(WidgetId::default_id(), "Test").with_theme(Theme::nord());
    let plane = cb.render(rect(0, 0, 20, 3));

    let bracket_chars: Vec<_> = plane.cells.iter().filter(|c| c.char == '[' || c.char == ']').collect();
    assert!(!bracket_chars.is_empty(), "checkbox should render bracket chars");
}

// === Verify visible cells have proper fg/bg ===

#[test]
fn test_button_text_cells_have_non_reset_fg() {
    let btn = Button::new("OK").with_theme(Theme::one_dark());
    let plane = btn.render(rect(0, 0, 10, 3));

    for cell in &plane.cells {
        if cell.char != ' ' {
            assert!(
                !matches!(cell.fg, Color::Reset),
                "visible cell with char '{}' should not have Reset fg",
                cell.char
            );
        }
    }
}

#[test]
fn test_toggle_text_cells_have_non_reset_fg() {
    let mut t = Toggle::new(WidgetId::default_id(), "Switch").with_theme(Theme::rose_pine());
    t.toggle();
    let plane = t.render(rect(0, 0, 30, 3));

    for cell in &plane.cells {
        if cell.char != ' ' {
            assert!(
                !matches!(cell.fg, Color::Reset),
                "visible cell should not have Reset fg"
            );
        }
    }
}
