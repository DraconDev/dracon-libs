//! TextEditor example — demonstrates TextEditor widget in App context.
//!
//! Opens a file and displays it in an editable text editor within the App.
//! Keyboard: type to edit, arrows to navigate, Ctrl+G for goto line,
//!           Ctrl+F to search, Ctrl+S to save.
//!
//! Usage:
//!     cargo run --example text_editor_demo -- [FILE_PATH]

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::WidgetId;
use dracon_terminal_engine::framework::widgets::TextEditorAdapter;
use dracon_terminal_engine::widgets::editor::TextEditor;
use std::path::PathBuf;
use ratatui::layout::Rect;

fn main() -> std::io::Result<()> {
    let theme = Theme::cyberpunk();

    let file_path = std::env::args().nth(1).map(PathBuf::from);

    let mut editor = if let Some(ref path) = file_path {
        if path.exists() {
            TextEditor::open(path).unwrap_or_else(|_| TextEditor::with_content(""))
        } else {
            TextEditor::with_content("")
        }
    } else {
        TextEditor::with_content("// Start typing...\n")
    };

    editor.with_show_line_numbers(true);
    editor.with_indent_guides(true);
    editor.with_status_bar(true);
    if let Some(ref path) = file_path {
        if let Some(ext) = path.extension() {
            if let Some(ext_str) = ext.to_str() {
                editor.with_language(ext_str);
            }
        }
    }

    let mut app = App::new()?
        .title("TextEditor Demo")
        .fps(30)
        .theme(theme);

    let adapter = TextEditorAdapter::new(WidgetId::new(1), editor);
    // Store the widget ID so we can resize it later
    let _widget_id = app.add_widget(Box::new(adapter), Rect::new(0, 0, 80, 24));

    app.run(move |ctx| {
        // Terminal resize is handled by the framework; the widget's area
        // will be updated automatically via set_area on the next render cycle.
        ctx.hide_cursor().ok();
    })
}
